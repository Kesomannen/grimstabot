use anyhow::{Context, Result};
use serde::Deserialize;

use crate::AppState;

use super::Ingredient;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Response {
    results: Vec<Product>,
    //sorts: Vec<Sort>,
    //pagination: Pagination,
    //facets: Vec<Facet>,
    //category_breadcrumbs: Option<CategoryInfo>,
    //category_info: CategoryInfo,
    //super_categories: Vec<CategoryInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Product {
    potential_promotions: Vec<Promotion>,
    inactive_potential_promotions: Vec<Promotion>,
    price_value: f64,
    price: String,
    image: Image,
    thumbnail: Image,
    code: String,
    name: String,
    compare_price: String,
    compare_price_unit: String,
    product_basket_type: ProductBasketType,
    price_unit: String,
    price_no_unit: String,
    online: bool,
    out_of_stock: bool,
    add_to_cart_disabled: bool,
    display_volume: String,
    manufacturer: String,
    labels: Vec<String>,
    product_line2: String,
    pickup_product_line2: String,
    savings_amount: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Promotion {
    code: String,
    applied: bool,
    campaign_type: String,
    promotion_type: String,
    main_product_code: String,
    product_codes: Vec<String>,
    qualifying_count: u32,
    price: Option<Price>,
    lowest_historical_price: Option<Price>,
    promotion_theme: Option<PromotionTheme>,
    condition_label: Option<String>,
    reward_label: Option<String>,
    compare_price: Option<String>,
    priority: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Price {
    currency_iso: String,
    value: f64,
    price_type: String,
    formatted_value: String,
    min_quantity: Option<u32>,
    max_quantity: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PromotionTheme {
    code: String,
    visible: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Image {
    image_type: String,
    format: String,
    url: String,
    alt_text: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProductBasketType {
    code: String,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Sort {
    code: String,
    name: String,
    selected: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Pagination {
    page_size: u32,
    current_page: u32,
    sort: String,
    number_of_pages: u32,
    total_number_of_results: u32,
    all_products_in_categories_count: u32,
    all_products_in_search_count: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Facet {
    code: String,
    name: String,
    priority: i32,
    category: bool,
    multi_select: bool,
    visible: bool,
    values: Vec<FacetValue>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FacetValue {
    code: String,
    name: String,
    count: u32,
    selected: bool,
    query: FacetQuery,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FacetQuery {
    url: String,
    query: FacetQueryValue,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FacetQueryValue {
    value: String,
    filter_queries: Vec<String>,
    search_query_context: Option<String>,
    search_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CategoryInfo {
    code: String,
    name: String,
    url: String,
    parent_category_name: Option<String>,
    sequence: u32,
    description: Option<String>,
    image: Option<String>,
    seo_title: Option<String>,
    seo_description: Option<String>,
}

async fn get_products_raw(
    category: &str,
    count: u32,
    base_url: &str,
    http: &reqwest::Client,
) -> Result<Vec<Product>> {
    let url = format!("https://{base_url}/c/{category}?page=0&size={count}");

    let text = http
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let res: Response = serde_json::from_str(&text)?;

    Ok(res.results)
}

async fn get_products(
    amount: f64,
    category: &str,
    base_url: &str,
    state: &AppState,
) -> Result<impl Iterator<Item = super::Product>> {
    let result = get_products_raw(category, 30, base_url, &state.http)
        .await?
        .into_iter()
        .map(move |product| {
            let comparative_price: f64 = product
                .compare_price
                .split_once(" ")
                .map(|(a, _)| a)
                .unwrap_or(&product.compare_price)
                .replace(',', ".")
                .parse()
                .context("failed to parse compare price")?;

            let comparative_price_text = format!("kr/{}", product.compare_price_unit);

            let url = format!(
                "https://{base_url}/produkt/{}-{}",
                product
                    .name
                    .replace('&', "och")
                    .replace('%', "procent")
                    .replace('ö', "o")
                    .replace(' ', ""),
                product.code
            );

            Ok(super::Product {
                url,
                name: product.name,
                manufacturer_name: product.manufacturer,
                comparative_price,
                comparative_price_text,
                price: comparative_price * amount,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(result.into_iter())
}

pub async fn get_willys_products(
    ingredient: &Ingredient,
    state: &AppState,
) -> Result<impl Iterator<Item = super::Product>> {
    get_products(
        ingredient.amount,
        &ingredient.willys_category_name,
        "www.willys.se",
        state,
    )
    .await
}

pub async fn get_hemkop_products(
    ingredient: &Ingredient,
    state: &AppState,
) -> Result<impl Iterator<Item = super::Product>> {
    Ok(get_products(
        ingredient.amount,
        &ingredient.hemkop_category_name,
        "www.hemkop.se",
        state,
    )
    .await?
    .filter(|product| !product.name.to_lowercase().contains("raps")))
}
