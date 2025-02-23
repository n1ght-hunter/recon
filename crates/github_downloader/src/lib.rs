pub mod models;

pub fn download(url: &str) {}

#[derive(Debug)]
pub enum DownloadObject {
    File {
        name: String,
        path: String,
        url: String,
    },
    Dir {
        name: String,
        path: String,
        url: String,
        children: Vec<DownloadObject>,
    },
}

pub fn list(url: &str) -> DownloadObject {
    let name = get_root_name(url);
    let objects = get_objects(url)
        .into_iter()
        .map(map_to_download_object)
        .collect::<Vec<DownloadObject>>();

    DownloadObject::Dir {
        name: name.clone(),
        path: name,
        url: url.to_string(),
        children: objects,
    }
}

fn get_root_name(url: &str) -> String {
    let parts = url.split("/").collect::<Vec<&str>>();
    let name = parts[parts.len() - 1].to_string();

    if name == "contents" {
        parts[parts.len() - 2].to_string()
    } else {
        name
    }
}

fn map_to_download_object(object: models::Object) -> DownloadObject {
    if object.type_field == models::FieldType::File {
        DownloadObject::File {
            path: object.path,
            name: object.name,
            url: object.download_url.unwrap(),
        }
    } else {
        DownloadObject::Dir {
            children: get_objects(&object.url)
                .into_iter()
                .map(map_to_download_object)
                .collect(),
            name: object.name,
            path: object.path,
            url: object.url,
        }
    }
}

fn get_objects(url: &str) -> models::Objects {
    let res = ureq::get(url)
        .call()
        .unwrap()
        .into_json::<models::Objects>()
        .unwrap();
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map_to_vec_strings(object: DownloadObject) -> Vec<String> {
        match object {
            DownloadObject::File {  path, .. } => vec![path],
            DownloadObject::Dir { path, children, .. } => {
                let mut names = vec![path];
                names.append(&mut children.into_iter().flat_map(map_to_vec_strings).collect::<Vec<_>>());
                names
            }
        }
    }

    #[test]
    fn test_get_objects() {
        let url = "https://api.github.com/repos/TsudaKageyu/minhook/contents/src";
        let output = list(url);

        let names = map_to_vec_strings(output);
        println!("{:?}", names);
    }
}
