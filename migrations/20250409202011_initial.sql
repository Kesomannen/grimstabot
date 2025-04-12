CREATE TABLE reports (
    id SERIAL PRIMARY KEY,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    report_id INTEGER NOT NULL REFERENCES reports(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    manufacturer_name TEXT NOT NULL,
    comparative_price FLOAT NOT NULL,
    comparative_price_text TEXT NOT NULL,
    url TEXT NOT NULL,
    store TEXT NOT NULL
);
