use std::{cmp::Ordering, collections::HashMap, fmt::Display, future::Future};

use anyhow::{anyhow, Result};
use chrono::Utc;
use itertools::Itertools;
use uuid::Uuid;

use crate::AppState;

mod coop;
mod ica;
pub mod plot;
pub mod update;

#[derive(Debug)]
pub struct Ingredient {
    pub id: i64,
    pub name: String,
    pub amount: f64,
    pub coop_id: i64,
    pub ica_category_name: String,
}

#[derive(Debug)]
pub struct Product {
    pub name: String,
    pub manufacturer_name: String,
    pub comparative_price: f64,
    pub comparative_price_text: String,
    pub url: String,
}

impl Product {
    pub fn price(&self, ingredient: &Ingredient) -> f64 {
        self.comparative_price * ingredient.amount
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Store {
    Coop,
    Ica,
}

impl Display for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Store::Coop => write!(f, "Coop"),
            Store::Ica => write!(f, "ICA"),
        }
    }
}

impl Store {
    pub fn id(&self) -> &'static str {
        match self {
            Store::Coop => "coop",
            Store::Ica => "ica",
        }
    }
}

pub struct Report {
    pub ingredients: HashMap<i64, Ingredient>,
    pub stores: HashMap<Store, HashMap<i64, Product>>,
}

impl Report {
    pub fn products_by_ingredient<'a>(
        &'a self,
        ingredient: &'a Ingredient,
    ) -> impl Iterator<Item = (Store, &'a Product)> + 'a {
        self.stores
            .iter()
            .map(|(store, products)| (*store, &products[&ingredient.id]))
    }

    pub fn cheapest(&self) -> impl Iterator<Item = (&Ingredient, (Store, &Product))> {
        self.ingredients.values().map(|ingredient| {
            (
                ingredient,
                self.products_by_ingredient(ingredient)
                    .min_by(|(_, a), (_, b)| a.comparative_price.total_cmp(&b.comparative_price))
                    .unwrap(),
            )
        })
    }
}

pub async fn create_report(state: &AppState) -> Result<Report> {
    let ingredients = sqlx::query_as!(Ingredient, "SELECT * FROM ingredients")
        .fetch_all(&state.db)
        .await?;

    let mut stores = HashMap::new();

    stores.insert(
        Store::Coop,
        create_store_report(coop::get_products, &ingredients, state).await?,
    );
    stores.insert(
        Store::Ica,
        create_store_report(ica::get_products, &ingredients, state).await?,
    );

    let ingredients = ingredients
        .into_iter()
        .map(|item| (item.id, item))
        .collect();

    let reports = Report {
        stores,
        ingredients,
    };

    Ok(reports)
}

async fn create_store_report<'a, F, R, Fut>(
    reporter: F,
    ingredients: &'a [Ingredient],
    state: &'a AppState,
) -> Result<HashMap<i64, Product>>
where
    F: Fn(&'a Ingredient, &'a AppState) -> Fut,
    Fut: Future<Output = Result<R>>,
    R: Iterator<Item = Product>,
{
    let mut result = HashMap::new();
    for ingredient in ingredients {
        let product = reporter(ingredient, state)
            .await?
            .filter(|product| product.name.starts_with(&ingredient.name))
            .sorted_by(|a, b| {
                a.comparative_price
                    .partial_cmp(&b.comparative_price)
                    .unwrap_or(Ordering::Equal)
            })
            .next()
            .ok_or_else(|| anyhow!("no products found"))?;

        result.insert(ingredient.id, product);
    }
    Ok(result)
}

pub fn create_embed(report: &Report) -> serenity::all::CreateEmbed {
    let products = report
        .cheapest()
        .map(|(ingredient, (store, product))| {
            (ingredient, store, product, product.price(ingredient))
        })
        .collect_vec();
    let total_price: f64 = products.iter().map(|(_, _, _, price)| price).sum();

    let fields = products
        .iter()
        .sorted_by(|(_, _, _, a), (_, _, _, b)| a.total_cmp(&b).reverse())
        .map(|(ingredient, store, product, price)| {
            (
                format!("{} `{:0.1}kr`", ingredient.name, price),
                format!(
                    "{} [{} {}]({}) ({}{})",
                    store,
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

pub async fn save_report(report: &Report, state: &AppState) -> Result<()> {
    let mut tx = state.db.begin().await?;

    let report_id = sqlx::query!("INSERT INTO reports DEFAULT VALUES RETURNING id")
        .fetch_one(&mut *tx)
        .await?
        .id;

    for (store, products) in &report.stores {
        for (ingredient_id, product) in products {
            let Product {
                name,
                manufacturer_name,
                comparative_price,
                comparative_price_text,
                url,
                ..
            } = product;

            let store_name = store.id();

            sqlx::query!(
                "INSERT INTO products 
                (report_id, ingredient_id, name, manufacturer_name, comparative_price, comparative_price_text, url, store)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                report_id,
                ingredient_id,
                name,
                manufacturer_name,
                comparative_price,
                comparative_price_text,
                url,
                store_name
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;

    Ok(())
}
