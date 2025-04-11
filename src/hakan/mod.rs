use anyhow::Result;
use chrono::Utc;
use itertools::Itertools;

use crate::AppState;

mod coop;
pub mod plot;
pub mod update;

#[derive(Debug)]
pub struct Ingredient {
    pub id: i64,
    pub name: String,
    pub amount: f64,
    pub coop_id: i64,
}

#[derive(Debug)]
pub struct Product {
    pub ingredient: Ingredient,
    pub name: String,
    pub manufacturer_name: String,
    pub comparative_price: f64,
    pub comparative_price_text: String,
    pub url: String,
}

impl Product {
    pub fn price(&self) -> f64 {
        self.comparative_price * self.ingredient.amount
    }
}

pub async fn get_products(state: &AppState) -> Result<Vec<Product>> {
    let ingredients = sqlx::query_as!(Ingredient, "SELECT * FROM ingredients")
        .fetch_all(&state.db)
        .await?;

    let mut result: Vec<Product> = Vec::new();

    for ingredient in ingredients {
        let product = coop::get_cheapest_product(state, &ingredient).await?;

        let url = product.url();

        let coop::Product {
            name,
            manufacturer_name,
            comparative_price_text,
            comparative_price,
            ..
        } = product;

        result.push(Product {
            ingredient,
            name,
            manufacturer_name,
            comparative_price,
            comparative_price_text,
            url,
        });
    }

    Ok(result)
}

pub fn create_embed(products: &[Product]) -> serenity::all::CreateEmbed {
    let total_price: f64 = products.iter().map(|product| product.price()).sum();

    let fields = products
        .iter()
        .sorted_by(|a, b| a.price().total_cmp(&b.price()).reverse())
        .map(|product| {
            (
                format!("{} `{:0.1}kr`", product.ingredient.name, product.price()),
                format!(
                    "[{} {}]({}) ({}{})",
                    product.manufacturer_name,
                    product.name,
                    product.url,
                    product.comparative_price,
                    product.comparative_price_text,
                ),
                false,
            )
        });

    serenity::all::CreateEmbed::new()
        .title(format!("📈 Håkanbörsen 📈"))
        .color(serenity::all::Color::DARK_GREEN)
        .description(format!(
            "<t:{}>\n# `{total_price:0.3}kr`",
            Utc::now().timestamp()
        ))
        .fields(fields)
}
