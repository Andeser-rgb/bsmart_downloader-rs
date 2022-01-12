use aes::Aes128;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use lopdf::{Document, Object, ObjectId};
use reqwest::header;
use serde_json::Value;
use std::collections::BTreeMap;
use std::{io, io::Write, str::FromStr};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print!("Input \"_bsw_session_v1_production\" cookie: ");
    io::stdout().flush().unwrap();
    let mut session_cookie = String::new();
    io::stdin()
        .read_line(&mut session_cookie)
        .expect("Failed to read the line");

    let session_cookie = session_cookie.trim();
    let header_value = format!("_bsw_session_v1_production={}", session_cookie);
    let mut request_headers = header::HeaderMap::new();
    request_headers.insert(
        header::COOKIE,
        header::HeaderValue::from_str(&header_value)?,
    );

    let client = reqwest::blocking::ClientBuilder::new()
        .default_headers(request_headers)
        .cookie_store(true)
        .build()?;

    let user = client
        .get("https://www.bsmart.it/api/v5/user")
        .send()?
        .bytes()?;
    let auth_token: Value = serde_json::from_slice(&user)?;

    let mut request_headers = header::HeaderMap::new();
    request_headers.insert(
        header::HeaderName::from_str("auth_token").unwrap(),
        header::HeaderValue::from_str(&auth_token["auth_token"].as_str().unwrap())?,
    );
    let client = reqwest::blocking::ClientBuilder::new()
        .default_headers(request_headers)
        .cookie_store(true)
        .build()?;
    let books = client
        .get("https://www.bsmart.it/api/v6/books?page_thumb_size=medium&per_page=25000")
        .send()?
        .bytes()?;

    let books: Value = serde_json::from_slice(&books)?;
    let books = books.as_array().unwrap();

    if books.len() == 0 {
        println!("There are no books in your library!");
        return Ok(());
    } else {
        println!("Book list: ");
        for value in books {
            println!("{} {}", value["id"], value["title"]);
        }
    }

    print!("Enter book id: ");
    io::stdout().flush().unwrap();
    let mut book_id = String::new();
    io::stdin()
        .read_line(&mut book_id)
        .expect("Failed to read the line");

    let book = client
        .get(format!(
            "https://www.bsmart.it/api/v6/books/by_book_id/{}",
            book_id
        ))
        .send()?
        .bytes()?;
    let book: Value = serde_json::from_slice(&book)?;
    let mut page = 1;
    let mut info = Vec::new();
    loop {
        let tempinfo = client
            .get(format!(
                "https://api.bsmart.it/api/v5/books/{}/{}/resources?per_page=500&page={}",
                book_id, book["current_edition"]["revision"], page
            ))
            .send()?
            .bytes()?;
        let tempinfo: Value = serde_json::from_slice(&tempinfo)?;
        let tempinfo = &**tempinfo.as_array().unwrap();
        info.extend_from_slice(tempinfo);
        if tempinfo.len() < 500 {
            break;
        }
        page += 1;
    }

    println!("Downloading pages");
    let mut pages = Vec::new();

    for (i, el) in info.iter().enumerate() {
        print!("{}[2J", 27 as char);
        println!("Progress: {:0.2}%", i as f64 / info.len() as f64 * 100.0);
        let assets = el["assets"].as_array().unwrap();
        for asset in assets {
            if asset["use"] != "page_pdf" {
                continue;
            }
            let url = asset["url"].as_str().unwrap();
            let mut page_data = download_and_decrypt(url);
            for _ in 0..3 {
                match page_data {
                    Ok(_) => break,
                    Err(e) => {
                        println!("Download of the file at url '{}' failed: {}", url, e);
                        println!("Trying again");
                        page_data = download_and_decrypt(url);
                    }
                }
            }
            let page_data = match page_data {
                Ok(s) => s,
                Err(e) => {
                    println!("Download of the file at url '{}' failed: {}", url, e);
                    println!("The file will be ignored");
                    continue;
                }
            };
            let pdf_page = Document::load_mem(&page_data).unwrap();
            pages.push(pdf_page);
        }
    }
    let mut document = merge_pdf(&mut pages)?;

    document
        .save(format!("{}-{}.pdf", book["id"], book["title"]))
        .unwrap();

    Ok(())
}

const KEY: &'static [u8] = &[
    30, 0, 184, 152, 115, 19, 157, 33, 4, 237, 80, 26, 139, 248, 104, 155,
];

fn download_and_decrypt(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let file: &[u8] = &reqwest::blocking::get(url)?.bytes()?;
    let file = file.into_iter().map(|&x| x).collect::<Vec<u8>>();

    let start = file
        .windows(5)
        .position(|window| window == b"start")
        .ok_or("Error, file not valid: 'start' not found")?
        + 6;
    let start_end = file
        .windows(4)
        .skip(start)
        .position(|window| window == b"path")
        .ok_or("Error, file not valid: 'start_end' not found")?
        + start
        - 1;
    let start_position = file[start..start_end]
        .iter()
        .map(|&x| x as i32)
        .rev()
        .enumerate()
        .reduce(|(_, a), (i, c)| (0, a + c * 256i32.pow(i as u32)))
        .ok_or("Error: Cannot infer starting position")?
        .1 as usize;

    let first_part = &mut file.clone()[256..start_position];
    let second_part = &file[start_position..];
    type Aes128Cbc = Cbc<Aes128, Pkcs7>;
    let cipher = Aes128Cbc::new_from_slices(KEY, &first_part[..16])?;
    let decrypted_first_part = cipher.decrypt(first_part[16..].as_mut())?;
    let mut result = Vec::with_capacity(decrypted_first_part.len() + second_part.len());
    result.extend_from_slice(decrypted_first_part);
    result.extend_from_slice(second_part);
    Ok(result)
}
fn merge_pdf(pages: &mut [Document]) -> Result<Document, Box<dyn std::error::Error>> {
    let mut documents_pages = BTreeMap::new();
    let mut documents_objects = BTreeMap::new();
    let mut document = Document::new();
    let mut max_id = 1;
    for page in pages {
        page.renumber_objects_with(max_id);
        max_id = page.max_id + 1;
        documents_pages.extend(
            page.get_pages()
                .into_iter()
                .map(|(_, object_id)| (object_id, page.get_object(object_id).unwrap().to_owned()))
                .collect::<BTreeMap<ObjectId, Object>>(),
        );
        documents_objects.extend(page.objects.clone());
    }
    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    // Process all objects except "Page" type
    for (object_id, object) in documents_objects.iter() {
        // We have to ignore "Page" (as are processed later), "Outlines" and "Outline" objects
        // All other objects should be collected and inserted into the main Document
        match object.type_name().unwrap_or("") {
            "Catalog" => {
                // Collect a first "Catalog" object and use it for the future "Pages"
                catalog_object = Some((
                    if let Some((id, _)) = catalog_object {
                        id
                    } else {
                        *object_id
                    },
                    object.clone(),
                ));
            }
            "Pages" => {
                // Collect and update a first "Pages" object and use it for the future "Catalog"
                // We have also to merge all dictionaries of the old and the new "Pages" object
                if let Ok(dictionary) = object.as_dict() {
                    let mut dictionary = dictionary.clone();
                    if let Some((_, ref object)) = pages_object {
                        if let Ok(old_dictionary) = object.as_dict() {
                            dictionary.extend(old_dictionary);
                        }
                    }

                    pages_object = Some((
                        if let Some((id, _)) = pages_object {
                            id
                        } else {
                            *object_id
                        },
                        Object::Dictionary(dictionary),
                    ));
                }
            }
            "Page" => {}     // Ignored, processed later and separately
            "Outlines" => {} // Ignored, not supported yet
            "Outline" => {}  // Ignored, not supported yet
            _ => {
                document.objects.insert(*object_id, object.clone());
            }
        }
    }

    // If no "Pages" found abort
    if pages_object.is_none() {
        println!("Pages root not found.");
        return Err(lopdf::Error::ObjectNotFound.into());
    }

    // Iter over all "Page" and collect with the parent "Pages" created before
    for (object_id, object) in documents_pages.iter() {
        if let Ok(dictionary) = object.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Parent", pages_object.as_ref().unwrap().0);

            document
                .objects
                .insert(*object_id, Object::Dictionary(dictionary));
        }
    }

    // If no "Catalog" found abort
    if catalog_object.is_none() {
        println!("Catalog root not found.");
        return Err(lopdf::Error::ObjectNotFound.into());
    }

    let catalog_object = catalog_object.unwrap();
    let pages_object = pages_object.unwrap();

    // Build a new "Pages" with updated fields
    if let Ok(dictionary) = pages_object.1.as_dict() {
        let mut dictionary = dictionary.clone();

        // Set new pages count
        dictionary.set("Count", documents_pages.len() as u32);

        // Set new "Kids" list (collected from documents pages) for "Pages"
        dictionary.set(
            "Kids",
            documents_pages
                .into_iter()
                .map(|(object_id, _)| Object::Reference(object_id))
                .collect::<Vec<_>>(),
        );

        document
            .objects
            .insert(pages_object.0, Object::Dictionary(dictionary));
    }

    // Build a new "Catalog" with updated fields
    if let Ok(dictionary) = catalog_object.1.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Pages", pages_object.0);
        dictionary.remove(b"Outlines"); // Outlines not supported in merged PDFs

        document
            .objects
            .insert(catalog_object.0, Object::Dictionary(dictionary));
    }

    document.trailer.set("Root", catalog_object.0);

    // Update the max internal ID as wasn't updated before due to direct objects insertion
    document.max_id = document.objects.len() as u32;

    // Reorder all new Document objects
    document.renumber_objects();

    //Set any Bookmarks to the First child if they are not set to a page
    document.adjust_zero_pages();

    //Set all bookmarks to the PDF Object tree then set the Outlines to the Bookmark content map.
    if let Some(n) = document.build_outline() {
        if let Ok(x) = document.get_object_mut(catalog_object.0) {
            if let Object::Dictionary(ref mut dict) = x {
                dict.set("Outlines", Object::Reference(n));
            }
        }
    }

    document.compress();
    Ok(document)
}
