use std::cmp::Ordering;

use anyhow::{bail, Result};
use chrono::Utc;
use convert_case::{Case, Casing};
use itertools::Itertools;

use crate::Db;

mod coop;
pub mod plot;

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

pub async fn get_products(db: &Db) -> Result<Vec<Product>> {
    let http = reqwest::Client::builder()
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:137.0) Gecko/20100101 Firefox/137.0",
        )
        .build()?;

    let ingredients = sqlx::query_as!(Ingredient, "SELECT * FROM ingredients")
        .fetch_all(db)
        .await?;

    let mut result: Vec<Product> = Vec::new();

    for ingredient in ingredients {
        let products = coop::get_products(
            &http,
            ingredient.coop_id as u32,
            10,
            vec![coop::SortBy {
                order: coop::SortOrder::Descending,
                attribute_name: "popularity".into(),
            }],
        )
        .await?;

        let Some(product) = products
            .into_iter()
            .filter(|product| product.name.starts_with(&ingredient.name))
            .sorted_by(|a, b| {
                a.comparative_price
                    .partial_cmp(&b.comparative_price)
                    .unwrap_or(Ordering::Equal)
            })
            .next()
        else {
            bail!("failed to find a product for {}", ingredient.name)
        };

        let mut categories = Vec::new();
        let mut category = product.nav_categories.into_iter().next().unwrap();
        loop {
            categories.push(category.name);
            match category.super_categories.into_iter().next() {
                Some(cat) => category = cat,
                None => break,
            }
        }

        let mut url = "https://coop.se/handla/varor/".to_string();
        for category in categories.into_iter().rev() {
            url.push_str(&category.replace('&', "").to_case(Case::Kebab));
            url.push('/');
        }

        url.push_str(&product.name.to_case(Case::Kebab));
        url.push('-');
        url.push_str(&product.id.to_string());

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
