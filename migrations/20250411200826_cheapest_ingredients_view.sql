DROP VIEW IF EXISTS cheapest_products;

CREATE VIEW cheapest_products AS
WITH ranked_products AS (
    SELECT
  		products.id,
      products.report_id,
      products.name,
      products.manufacturer_name,
      products.comparative_price,
      products.comparative_price_text,
      products.url,
      products.store,
      products.ingredient_id,
      (products.comparative_price * ingredients.amount) AS price,
      ROW_NUMBER() OVER (
        PARTITION BY products.report_id, products.ingredient_id
        ORDER BY (products.comparative_price * ingredients.amount)
      ) AS rn
    FROM products
    JOIN ingredients
      ON ingredients.id = products.ingredient_id
  )
SELECT *
FROM ranked_products
WHERE rn = 1;
  