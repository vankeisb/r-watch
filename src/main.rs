
mod bamboo;
mod build_status;

use std::env;


// enum BuildDef {
//     Bamboo(bamboo::BambooDef),
// }

// impl BuildDef {
//     async fn fetch(&self) -> build_status::BuildStatus {
//         match self {
//             BuildDef::Bamboo(b) => {
//                 b.fetch().await
//             }
//         }
//     }
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

   
    Ok(())
}