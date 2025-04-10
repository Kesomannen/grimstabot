use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub results: Results,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Results {
    pub count: u32,
    pub facets: Vec<serde_json::Value>,
    pub items: Vec<Product>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    pub id: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub ean: String,
    pub name: String,
    pub manufacturer_name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub list_of_ingredients: Option<String>,
    pub nav_categories: Vec<NavCategory>,
    pub accredited_tags: Option<Vec<AccreditedTag>>,
    pub max_storage_temperature: Option<String>,
    pub package_size: f64,
    pub package_size_information: String,
    pub package_size_unit: String,
    pub sales_price: f64,
    pub sales_price_data: PriceData,
    pub piece_price: f64,
    pub piece_price_data: PriceData,
    pub sales_unit: String,
    pub comparative_price_text: String,
    pub comparative_price: f64,
    pub comparative_price_data: PriceData,
    pub article_sold: bool,
    pub deposit: f64,
    pub deposit_data: PriceData,
    pub vat: Vat,
    pub animal_food_data: serde_json::Value,
    pub from_sweden: bool,
    pub available_online: bool,
    pub local_product: Option<LocalProduct>,
    pub is_magazine: bool,
    pub nutrient_basis: Option<NutrientBasis>,
    pub pharmaceutical_data: PharmaceuticalData,
    pub nutrient_links: Option<Vec<NutrientLink>>,
    pub nutrient_information: Option<Vec<NutrientInfo>>,
    pub country_of_origin_codes: Option<Vec<CountryCode>>,
    pub consumer_instructions: Option<ConsumerInstructions>,
    pub preparation_instructions: Option<String>,
    pub preparation_instructions_list: Option<Vec<String>>,
    pub sustainability_info: Option<Vec<SustainabilityInfo>>,
    pub sustainability_info_applicable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavCategory {
    pub code: String,
    pub name: String,
    pub super_categories: Vec<NavCategory>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccreditedTag {
    pub code: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub priority: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceData {
    pub b2c_price: f64,
    pub b2b_price: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vat {
    pub code: String,
    pub value: f64,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalProduct {
    pub code: String,
    pub description: String,
    pub image_url: String,
    pub priority: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NutrientBasis {
    pub quantity: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PharmaceuticalData {
    pub is_pharmaceutical: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NutrientLink {
    pub amount: Vec<String>,
    pub description: String,
    pub unit: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NutrientInfo {
    pub header: NutrientHeader,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NutrientHeader {
    pub nutrient_basis_quantity: f64,
    pub nutrient_basis_quantity_type: Option<String>,
    pub nutrient_basis_quantity_unit: Option<Unit>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Unit {
    pub code: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CountryCode {
    pub code: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsumerInstructions {
    pub storage_instructions: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SustainabilityInfo {
    pub product_score: Vec<SustainabilityScore>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SustainabilityScore {
    pub score: Option<f64>,
    pub param: String,
    pub param_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Request {
    attribute: RequestAttribute,
    results_options: ResultsOptions,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RequestAttribute {
    name: String,
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResultsOptions {
    skip: u32,
    take: u32,
    sort_by: Vec<SortBy>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SortBy {
    pub order: SortOrder,
    pub attribute_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SortOrder {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

const API_KEY: &str = "3becf0ce306f41a1ae94077c16798187";

pub async fn get_products(
    http: &reqwest::Client,
    category: u32,
    count: u32,
    sort_by: Vec<SortBy>,
) -> Result<Vec<Product>> {
    const URL: &str = "https://external.api.coop.se/personalization/search/entities/by-attribute?api-version=v1&store=251300&groups=CUSTOMER_PRIVATE";

    let req = Request {
        attribute: RequestAttribute {
            name: "categoryIds".into(),
            value: category.to_string(),
        },
        results_options: ResultsOptions {
            skip: 0,
            take: count,
            sort_by,
        },
    };

    let text = http
        .post(URL)
        .json(&req)
        .header("Ocp-Apim-Subscription-Key", API_KEY)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    //tokio::fs::write(
    //    format!(r"C:\Users\bobbo\Documents\coop_{}.json", category),
    //    &text,
    //)
    //.await?;

    let res: Response = serde_json::from_str(&text)?;

    Ok(res.results.items)
}
