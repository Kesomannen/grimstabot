use std::{cmp::Ordering, collections::HashMap, fmt::Display, future::Future};

use anyhow::{anyhow, Result};
use itertools::Itertools;
use tracing::{error, info};

use crate::AppState;

mod axfood;
mod coop;
mod db;
mod ica;
mod mathem;
pub mod plot;
pub mod update;

#[derive(Debug)]
pub struct Ingredient {
    pub id: i32,
    pub name: String,
    pub aliases: String,
    pub amount: f64,
    pub coop_id: i32,
    pub ica_category_name: String,
    pub willys_category_name: String,
    pub hemkop_category_name: String,
    pub mathem_category_name: String,
}

#[derive(Debug)]
pub struct Product {
    pub name: String,
    pub manufacturer_name: String,
    pub comparative_price: f64,
    pub comparative_price_text: String,
    pub url: String,
    pub price: f64,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Store {
    Coop,
    Ica,
    Willys,
    Hemkop,
    Mathem,
}

impl Display for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Store::Coop => write!(f, "Coop"),
            Store::Ica => write!(f, "ICA"),
            Store::Willys => write!(f, "Willy:s"),
            Store::Hemkop => write!(f, "Hemköp"),
            Store::Mathem => write!(f, "Mathem"),
        }
    }
}

impl Store {
    pub fn id(&self) -> &'static str {
        match self {
            Store::Coop => "coop",
            Store::Ica => "ica",
            Store::Willys => "willys",
            Store::Hemkop => "hemkop",
            Store::Mathem => "mathem",
        }
    }
}

pub struct Report {
    pub ingredients: HashMap<i32, Ingredient>,
    pub stores: HashMap<Store, HashMap<i32, Product>>,
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

    pub fn cheapest(&self) -> impl Iterator<Item = (&Ingredient, Store, &Product)> {
        self.ingredients.values().map(|ingredient| {
            let (store, product) = self
                .products_by_ingredient(ingredient)
                .min_by(|(_, a), (_, b)| a.comparative_price.total_cmp(&b.comparative_price))
                .unwrap();

            (ingredient, store, product)
        })
    }
}

pub async fn create_report(state: &AppState) -> Result<Report> {
    let ingredients = sqlx::query_as!(Ingredient, "SELECT * FROM ingredients")
        .fetch_all(&state.db)
        .await?;

    let mut stores = HashMap::new();

    let (coop, ica, willys, hemkop, mathem) = tokio::join!(
        create_store_report(coop::get_products, &ingredients, state),
        create_store_report(ica::get_products, &ingredients, state),
        create_store_report(axfood::get_willys_products, &ingredients, state),
        create_store_report(axfood::get_hemkop_products, &ingredients, state),
        create_store_report(mathem::get_products, &ingredients, state),
    );

    insert_store_report(&mut stores, Store::Coop, coop);
    insert_store_report(&mut stores, Store::Ica, ica);
    insert_store_report(&mut stores, Store::Willys, willys);
    insert_store_report(&mut stores, Store::Hemkop, hemkop);
    insert_store_report(&mut stores, Store::Mathem, mathem);

    let ingredients = ingredients
        .into_iter()
        .map(|item| (item.id, item))
        .collect();

    Ok(Report {
        stores,
        ingredients,
    })
}

async fn create_store_report<'a, F, R, Fut>(
    reporter: F,
    ingredients: &'a [Ingredient],
    state: &'a AppState,
) -> Result<HashMap<i32, Product>>
where
    F: Fn(&'a Ingredient, &'a AppState) -> Fut,
    Fut: Future<Output = Result<R>>,
    R: Iterator<Item = Product>,
{
    let mut result = HashMap::new();
    for ingredient in ingredients {
        let product = reporter(ingredient, state)
            .await?
            .filter(|product| {
                let first_word = product
                    .name
                    .split_once(' ')
                    .map(|(word, _)| word)
                    .unwrap_or(&product.name);

                first_word == ingredient.name
                    || ingredient
                        .aliases
                        .split(',')
                        .any(|alias| first_word == alias)
            })
            .sorted_by(|a, b| {
                a.comparative_price
                    .partial_cmp(&b.comparative_price)
                    .unwrap_or(Ordering::Equal)
            })
            .next()
            .ok_or_else(|| anyhow!("no products found for {}", ingredient.name))?;

        result.insert(ingredient.id, product);
    }
    Ok(result)
}

fn insert_store_report(
    map: &mut HashMap<Store, HashMap<i32, Product>>,
    store: Store,
    res: Result<HashMap<i32, Product>>,
) {
    match res {
        Ok(products) => {
            map.insert(store, products);
        }
        Err(err) => error!(store = store.id(), "failed to create report: {err:#}"),
    }
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
