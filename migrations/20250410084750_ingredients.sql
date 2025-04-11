CREATE TABLE ingredients (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL,
    amount FLOAT NOT NULL,
    coop_id INTEGER NOT NULL,
    ica_category_name TEXT NOT NULL
);

ALTER TABLE products
ADD COLUMN ingredient_id INTEGER NOT NULL REFERENCES ingredients(id);

INSERT INTO ingredients (name, amount, coop_id, ica_category_name)
VALUES
    ('Ägg', 8.0, 334710, 'mejeri-ost/ägg-jäst/e1be6c83-70ec-4da5-b86b-ce0aa3593abe'),
    ('Vetemjöl', 0.5, 47876, 'skafferi/bakning/mjöl/d3a2055c-2bc2-44fb-afea-ab8d79cd89c4'),
    ('Strösocker', 0.85, 334547, 'skafferi/bakning/sötning/72e5884a-6d3c-4c6b-b3bb-6c4712dee79b'),
    ('Kakao', 0.125, 334550, 'skafferi/bakning/choklad-kakao/7c37c70e-e047-4eca-be0d-94528a7460f0'),
    ('Smör', 0.4, 334720, 'mejeri-ost/smör-margarin/59a7915d-a2e7-48e8-b639-79dca73018b0');
