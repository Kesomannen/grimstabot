use std::{collections::HashMap, future::Future};

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
}

impl Store {
    pub fn as_str(&self) -> &'static str {
        match self {
            Store::Coop => "coop",
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

    let store_reporters = [(Store::Coop, coop::get_cheapest_product)];

    for (store, reporter) in store_reporters {
        let products = create_store_report(reporter, &ingredients, state).await?;
        stores.insert(store, products);
    }

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
    R: Into<Product>,
{
    let mut result = HashMap::new();
    for ingredient in ingredients {
        let product = reporter(ingredient, state).await?;
        result.insert(ingredient.id, product.into());
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
                    store.as_str(),
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
