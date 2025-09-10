use std::collections::HashMap;

use anyhow::{anyhow, bail, Context, Result};
use convert_case::Casing;
use itertools::Itertools;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::AppState;

use super::{Ingredient, Product};

const STORE_ID: u32 = 1003823;

const BASE_URL: &str = "https://handlaprivatkund.ica.se";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ProductsResponse {
    entities: Entities,
    result: IcaResult,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Entities {
    product: HashMap<String, IcaProduct>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct IcaResult {
    categories: Vec<Category>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PriceInfo {
    amount: String,
    currency: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct UnitPrice {
    label: String,
    original: Option<PriceInfo>,
    current: PriceInfo,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Price {
    original: Option<PriceInfo>,
    current: PriceInfo,
    unit: UnitPrice,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ImageInfo {
    src: String,
    description: String,
    fop_srcset: String,
    bop_srcset: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Offer {
    id: String,
    retailer_promotion_id: String,
    description: String,
    #[serde(rename = "type")]
    offer_type: OfferType,
    presentation_mode: PresentationMode,
    limit_reached: bool,
    required_product_quantity: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum OfferType {
    Offer,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum PresentationMode {
    Default,
    MuteStyle,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Size {
    value: String,
    uom: Option<String>,
    catch_weight: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CatchweightQuantity {
    value: String,
    uom: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Catchweight {
    min_quantity: CatchweightQuantity,
    typical_quantity: CatchweightQuantity,
    max_quantity: CatchweightQuantity,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Attribute {
    icon: String,
    label: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct IcaProduct {
    product_id: String,
    retailer_product_id: String,
    name: String,
    available: bool,
    max_quantity_reached: bool,
    price: Price,
    is_in_current_catalog: bool,
    is_in_product_list: bool,
    category_path: Vec<String>,
    brand: String,
    country_of_origin: Option<String>,
    image: ImageInfo,
    images: Vec<ImageInfo>,
    offers: Option<Vec<Offer>>,
    offer: Option<Offer>,
    size: Option<Size>,
    catchweight: Option<Catchweight>,
    attributes: Option<Vec<Attribute>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Category {
    id: String,
    name: String,
    full_url_path: String,
    retailer_category_id: String,
}

pub async fn get_products(
    ingredient: &Ingredient,
    state: &AppState,
) -> Result<impl Iterator<Item = Product>> {
    let url = format!(
        "{BASE_URL}/stores/{STORE_ID}/api/v6/products?sort=favorite&category={}",
        ingredient.ica_category_name
    );

    let result: ProductsResponse = state
        .http
        .get(url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:141.0) Gecko/20100101 Firefox/141.0",
        )
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let products = result.entities.product.into_iter().map(|(_, product)| {
        let comparative_price = product.price.unit.current.amount.parse()?;
        let price = product.price.current.amount.parse()?;

        let comparative_price_text = match product.price.unit.label.as_str() {
            "fop.price.per.kg" => "kr/kg",
            "fop.price.per.each" => "kr/st",
            label => bail!("unknown unit price label: {label}"),
        }
        .to_string();

        let url = format!(
            "{BASE_URL}/stores/{STORE_ID}/products/{}/{}",
            product.name.to_case(convert_case::Case::Kebab),
            product.retailer_product_id
        );

        Ok(Product {
            comparative_price,
            comparative_price_text,
            price,
            name: product.name,
            manufacturer_name: product.brand,
            url,
        })
    });

    products.collect::<Result<Vec<_>, _>>().map(Vec::into_iter)
}
