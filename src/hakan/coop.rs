use anyhow::Result;
use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};

use crate::AppState;

use super::Ingredient;

#[derive(Debug, Deserialize)]
struct Response {
    results: Results,
}

#[derive(Debug, Deserialize)]
struct Results {
    count: u32,
    facets: Vec<serde_json::Value>,
    items: Vec<Product>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Product {
    id: String,
    #[serde(rename = "type")]
    item_type: String,
    ean: String,
    name: String,
    manufacturer_name: String,
    description: Option<String>,
    image_url: Option<String>,
    list_of_ingredients: Option<String>,
    nav_categories: Vec<NavCategory>,
    accredited_tags: Option<Vec<AccreditedTag>>,
    max_storage_temperature: Option<String>,
    package_size: f64,
    package_size_information: String,
    package_size_unit: String,
    sales_price: f64,
    sales_price_data: PriceData,
    piece_price: f64,
    piece_price_data: PriceData,
    sales_unit: String,
    comparative_price_text: String,
    comparative_price: f64,
    comparative_price_data: PriceData,
    article_sold: bool,
    deposit: f64,
    deposit_data: PriceData,
    vat: Vat,
    animal_food_data: serde_json::Value,
    from_sweden: bool,
    available_online: bool,
    local_product: Option<LocalProduct>,
    is_magazine: bool,
    nutrient_basis: Option<NutrientBasis>,
    pharmaceutical_data: PharmaceuticalData,
    nutrient_links: Option<Vec<NutrientLink>>,
    nutrient_information: Option<Vec<NutrientInfo>>,
    country_of_origin_codes: Option<Vec<CountryCode>>,
    consumer_instructions: Option<ConsumerInstructions>,
    preparation_instructions: Option<String>,
    preparation_instructions_list: Option<Vec<String>>,
    sustainability_info: Option<Vec<SustainabilityInfo>>,
    sustainability_info_applicable: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NavCategory {
    code: String,
    name: String,
    super_categories: Vec<NavCategory>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccreditedTag {
    code: String,
    description: Option<String>,
    image_url: Option<String>,
    priority: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PriceData {
    b2c_price: f64,
    b2b_price: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Vat {
    code: String,
    value: f64,
    r#type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalProduct {
    code: String,
    description: String,
    image_url: String,
    priority: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NutrientBasis {
    quantity: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PharmaceuticalData {
    is_pharmaceutical: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NutrientLink {
    amount: Vec<String>,
    description: String,
    unit: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NutrientInfo {
    header: NutrientHeader,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NutrientHeader {
    nutrient_basis_quantity: f64,
    nutrient_basis_quantity_type: Option<String>,
    nutrient_basis_quantity_unit: Option<Unit>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Unit {
    code: String,
    value: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CountryCode {
    code: String,
    value: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConsumerInstructions {
    storage_instructions: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SustainabilityInfo {
    product_score: Vec<SustainabilityScore>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SustainabilityScore {
    score: Option<f64>,
    param: String,
    param_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Request {
    attribute: RequestAttribute,
    results_options: ResultsOptions,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RequestAttribute {
    name: String,
    value: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResultsOptions {
    skip: u32,
    take: u32,
    sort_by: Vec<SortBy>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SortBy {
    order: SortOrder,
    attribute_name: String,
}

#[derive(Debug, Serialize)]
enum SortOrder {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

const API_KEY: &str = "3becf0ce306f41a1ae94077c16798187";

async fn get_products_raw(
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

    let res: Response = serde_json::from_str(&text)?;

    Ok(res.results.items)
}

pub async fn get_products(
    ingredient: &Ingredient,
    state: &AppState,
) -> Result<impl Iterator<Item = super::Product>> {
    let amount = ingredient.amount;

    Ok(get_products_raw(
        &state.http,
        ingredient.coop_id as u32,
        10,
        vec![SortBy {
            order: SortOrder::Descending,
            attribute_name: "popularity".into(),
        }],
    )
    .await?
    .into_iter()
    .map(move |product| {
        let url = product.url();

        super::Product {
            url,
            name: product.name,
            manufacturer_name: product.manufacturer_name,
            comparative_price: product.comparative_price,
            comparative_price_text: product.comparative_price_text,
            price: product.comparative_price * amount,
        }
    }))
}

impl Product {
    fn url(&self) -> String {
        let mut categories = Vec::new();
        let mut current = self.nav_categories.iter().next().unwrap();
        loop {
            categories.push(&current.name);
            match current.super_categories.iter().next() {
                Some(cat) => current = &cat,
                None => break,
            }
        }

        let mut url = "https://coop.se/handla/varor/".to_string();
        for category in categories.into_iter().rev() {
            url.push_str(&category.replace('&', "").to_case(Case::Kebab));
            url.push('/');
        }

        url.push_str(&self.name.to_case(Case::Kebab));
        url.push('-');
        url.push_str(&self.id.to_string());

        url
    }
}
