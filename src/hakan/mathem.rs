use anyhow::{Context, Result};
use scraper::{Html, Selector};

use crate::AppState;

use super::{Ingredient, Product};

pub async fn get_products(
    ingredient: &Ingredient,
    state: &AppState,
) -> Result<impl Iterator<Item = Product>> {
    let url = format!(
        "https://www.mathem.se/se/categories/{}",
        ingredient.mathem_category_name
    );
    let html = state
        .http
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let product_selector = Selector::parse("article.k-card").unwrap();
    let name_selector = Selector::parse("a").unwrap();
    let manufacturer_selector = Selector::parse(".styles_ProductNameExtraTile__yZuNO").unwrap();
    let price_selector =
        Selector::parse("p.k-text-style.k-text-style--label-s.k-text-color--subdued").unwrap();

    let document = Html::parse_document(&html);

    let mut products = Vec::new();

    for product_ele in document.select(&product_selector) {
        let name_ele = product_ele
            .select(&name_selector)
            .next()
            .context("product name element is missing")?;

        let name = name_ele
            .text()
            .next()
            .context("name element text is missing")?
            .to_string();

        let manufacturer_name = product_ele
            .select(&manufacturer_selector)
            .next()
            .context("manufacturer element is missing")?
            .text()
            .next()
            .context("manufacturer element text is missing")?
            .split(' ')
            .next_back()
            .context("malformed manufacturer text")?
            .to_string();

        let (price, price_text) = product_ele
            .select(&price_selector)
            .next()
            .context("price element is missing")?
            .text()
            .next()
            .context("price element text is missing")?
            .split_once('\u{a0}')
            .context("malformed price text")?;

        let comparative_price: f64 = price
            .trim()
            .replace(',', ".")
            .parse()
            .context("invalid price number format")?;

        let comparative_price_text = price_text.replace('\u{2009}', "");

        if !comparative_price_text.contains('/') {
            continue;
        }

        let href = name_ele.attr("href").context("name is missing href")?;
        let url = format!("https://www.mathem.se{href}");

        products.push(Product {
            name,
            manufacturer_name,
            comparative_price,
            comparative_price_text,
            url,
            price: comparative_price * ingredient.amount,
        });
    }

    Ok(products.into_iter())
}
