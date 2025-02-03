use sysinfo::Disks;
use std::{fs::{self, File}, io::{BufReader, Error}, path::Path, thread::sleep, time::Duration};
use xml::{
    reader::{EventReader, XmlEvent},
    name::OwnedName,
};

fn main() -> Result<(), Error> {
    let path = loop {
        println!("Probing...");
        if let Some(path) = probe_for_device() {
            break path
        }
        sleep(Duration::from_secs_f64(1.0));
    };

    println!("Found Kobo device! path: {:?}", path);

    let path = path.join("Digital Editions").join("Annotations");
    
    if !path.is_dir() {
        panic!("Unable to access Kobo annotations directory!")
    }
    let files = find_annotation_files(&path);

    for filename in files {
        if let Ok(file) = File::open(&filename) {
            let file = BufReader::new(file);
            let parser = EventReader::new(file);
            let mut current_tag = String::from("");
            for e in parser {
                match e {
                    Ok(XmlEvent::StartElement { name: OwnedName { local_name: tagname, .. }, .. }) => {
                        current_tag = tagname;
                    }
                    Ok(XmlEvent::Characters(content)) => {
                        if current_tag == "text" {
                            println!("{:?}", content);
                        } else if current_tag == "title" {
                            println!("\n=== {} ===\n", content);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        break;
                    }
                    _ => {}
                }
            }
        }

    }

    Ok(())
}

fn find_annotation_files(dir: &Path) -> Vec<Box<Path>> {
    let mut files = vec![];
    if dir.is_dir() {
        if let Ok(listing) = fs::read_dir(dir) {
            for entry in listing {
                if let Ok(entry) = entry {
                    if entry.path().is_dir() {
                        files.append(&mut find_annotation_files(&entry.path()));
                    } else if is_annotation_file(&entry.path()) {
                        files.push(entry.path().into());
                    }
                }
            }
        }
    }
    return files;
}

fn is_annotation_file(file: &Path) -> bool {
    match file.extension() {
        Some(str) if str == "annot" => true,
        _ => false
    }
}

fn probe_for_device() -> Option<Box<Path>> {
    let disks = Disks::new_with_refreshed_list();
    for disk in disks.list() {
        if disk.name() == "KOBOeReader" {
            return Some(disk.mount_point().into())
        }
    }
    return None
}
