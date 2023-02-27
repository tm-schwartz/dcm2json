use dicom::core::dictionary::{DataDictionary, DictionaryEntry};
use dicom::core::header::DataElement;
use dicom::dictionary_std::StandardDataDictionary;
use dicom_object::mem::InMemElement;
use dicom_object::InMemDicomObject;
use dicom_object::OpenFileOptions;
use dicom_object::Tag;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

struct Seq {
    key: String,
    val: String,
}

fn get_name(elem: &InMemElement) -> String {
    let tag_alias = StandardDataDictionary
        .by_tag(elem.header().tag)
        .map(DictionaryEntry::alias)
        .unwrap_or("Unknown Attribute");
    return tag_alias.to_string();
}

fn get_name_from_tag(tag: Tag) -> String {
    let tag_alias = StandardDataDictionary
        .by_tag(tag)
        .map(DictionaryEntry::alias)
        .unwrap_or("Unknown Attribute");
    return tag_alias.to_string();
}

fn process_nests(elem: &DataElement<InMemDicomObject, Vec<u8>>) -> Seq {
    let mut tgs = Vec::new();
    let mut fin_vals: String = "".to_string();
    let mut strng_tgs: String = "".to_string();
    for el in elem.items().iter() {
        if el.is_empty() {
            continue;
        }
        for e in el.iter() {
            let fuck = e.tags().map(|x| e.element(x)).collect::<Vec<_>>();
            tgs.append(&mut e.tags().collect::<Vec<Tag>>());
            strng_tgs.push_str(
                &e.tags()
                    .map(|x| get_name_from_tag(x))
                    .collect::<Vec<String>>()
                    .join("%~%"),
            );
            for f in fuck.iter() {
                if f.to_owned().as_ref().unwrap().vr().to_string() == "SQ".to_string() {
                    let s = process_nests(f.as_ref().unwrap());

                    strng_tgs.push_str(&s.key);
                    strng_tgs.push_str("%~%");
                    fin_vals.push_str(&s.val);
                    fin_vals.push_str("%~%");
                    continue;
                }
                fin_vals.push_str(&f.to_owned().as_ref().unwrap().to_str().unwrap().to_string());
                fin_vals.push_str("%~%");
            }
        }
    }
    return Seq {
        key: strng_tgs,
        val: fin_vals,
    };
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = &mut args[1].to_string();
    let obj = OpenFileOptions::new()
        .read_until(dicom::dictionary_std::tags::PIXEL_DATA)
        .open_file(&file_path)
        .expect("File not found");
    let mut out_file = "".to_string();
    if args.len() == 3 {
        out_file.push_str(&args[2]);
    } else {
        file_path.push_str(".json");
        out_file.push_str(file_path);
    }

    let mut header_map = HashMap::new();

    for element in obj.iter() {
        let tag_al = get_name(element);

        if element.vr().to_string() == "SQ" {
            let seq_entries = process_nests(element);
            for (k, v) in seq_entries.key.split("%~%").zip(seq_entries.val.split("%~%"))
            {
                v.to_string().push('\n');
                header_map.insert(k.to_string(), v.to_string());

            }
        } else {
            element.value().to_str().unwrap().to_string().push('\n');
            header_map.insert(tag_al, element.value().to_str().unwrap().to_string());
        }
    }
    let path = Path::new(&out_file);
    let display = path.display();

    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    let j = serde_json::to_string_pretty(&header_map).expect("Error!!!");

    file.write_all(j.as_bytes())
        .expect("Could not write or convert to bytes!");
}
