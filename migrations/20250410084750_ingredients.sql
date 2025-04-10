CREATE TABLE ingredients (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL,
    amount FLOAT NOT NULL,
    coop_id INTEGER NOT NULL
);

ALTER TABLE products
ADD COLUMN ingredient_id INTEGER NOT NULL REFERENCES ingredients(id);

INSERT INTO ingredients (name, amount, coop_id)
VALUES
    ('Ägg', 8.0, 334710),
    ('Vetemjöl', 0.5, 47876),
    ('Strösocker', 0.85, 334547),
    ('Kakao', 0.125, 334550),
    ('Smör', 0.4, 334720);
