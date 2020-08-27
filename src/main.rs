use tokio::prelude::*;
#[macro_use]
extern crate vkapi;

use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let file = String::from_utf8(
        std::fs::read(
            args.get(1)
                .expect("Передайте путь к файлу как первый аргумент"),
        )
        .unwrap(),
    )
    .unwrap();
    let names: Vec<&str> = file.split("\n").collect();
    let token = args
        .get(2)
        .expect("Передайте access_token как второй аргумент")
        .to_owned();

    let mut vk_api = Arc::new(Mutex::new(vkapi::VK::new("5.103", "en", token)));

    let mut tasks = Vec::new();

    for name in names {
        let name = name.replace("\r", ""); // на Windows помимо \n нужно убирать еще и \r
        let vkapi_cloned = vk_api.clone();

        tasks.push(tokio::spawn(async move {
            let mut response = vkapi_cloned
                .lock()
                .await
                .request(
                    "utils.resolveScreenName",
                    &mut param! {"screen_name" => &name},
                )
                .await;

            if response.is_err(){
                // ждем секунду, так как словили бан
                tokio::time::delay_for(std::time::Duration::new(0, 300)).await;

                response = vkapi_cloned
                .lock()
                .await
                .request(
                    "utils.resolveScreenName",
                    &mut param! {"screen_name" => &name},
                )
                .await;
            }

            if response.unwrap()["response"].is_empty() {
                println!("Имя {} свободно!", name);
            }
        }));
    }

    futures::future::join_all(tasks).await; // ждем пока все таски будут закончены
}
