-- Sample SQL file with DDL and DML statements

CREATE SCHEMA inventory;

CREATE TABLE inventory.products (
    id          SERIAL PRIMARY KEY,
    name        VARCHAR(255) NOT NULL,
    price       NUMERIC(10, 2) NOT NULL,
    category    VARCHAR(100),
    stock       INTEGER DEFAULT 0,
    created_at  TIMESTAMP DEFAULT NOW()
);

CREATE TABLE inventory.orders (
    id          SERIAL PRIMARY KEY,
    product_id  INTEGER REFERENCES inventory.products(id),
    quantity    INTEGER NOT NULL,
    total_price NUMERIC(10, 2),
    ordered_at  TIMESTAMP DEFAULT NOW()
);

CREATE VIEW inventory.low_stock AS
    SELECT id, name, stock
    FROM inventory.products
    WHERE stock < 10
    ORDER BY stock ASC;

CREATE MATERIALIZED VIEW inventory.category_totals AS
    SELECT category, SUM(price) AS total
    FROM inventory.products
    GROUP BY category;

CREATE INDEX idx_products_category ON inventory.products (category);

CREATE SEQUENCE inventory.order_seq AS BIGINT INCREMENT 1 START 1000;

CREATE TRIGGER touch_orders
AFTER INSERT ON inventory.orders
FOR EACH ROW
EXECUTE FUNCTION inventory.notify_new_order();

CREATE FUNCTION inventory.calculate_total(qty INTEGER, unit_price NUMERIC)
RETURNS NUMERIC AS $$
BEGIN
    RETURN qty * unit_price;
END;
$$ LANGUAGE plpgsql;

-- NOTE: the arborium-sql grammar does not parse PL/pgSQL's `IF ... THEN ...
-- ELSE ... END IF;` procedural control flow (confirmed via real parse: it
-- produces ERROR nodes). Written with a CASE expression instead, which the
-- grammar parses cleanly and which is idiomatic plpgsql anyway.
CREATE FUNCTION inventory.reorder_needed(product_id INTEGER)
RETURNS BOOLEAN AS $$
DECLARE
    current_stock INTEGER;
BEGIN
    SELECT stock INTO current_stock
    FROM inventory.products
    WHERE id = product_id;

    RETURN CASE WHEN current_stock < 5 THEN TRUE ELSE FALSE END;
END;
$$ LANGUAGE plpgsql;

-- Query with JOIN and aggregation
SELECT
    p.name,
    COUNT(o.id) AS order_count,
    SUM(o.total_price) AS revenue
FROM inventory.products p
LEFT JOIN inventory.orders o ON p.id = o.product_id
WHERE p.category = 'electronics'
GROUP BY p.name
HAVING COUNT(o.id) > 0
ORDER BY revenue DESC;

-- CTE (WITH clause) feeding a window function
WITH recent_orders AS (
    SELECT *
    FROM inventory.orders
    WHERE ordered_at > NOW() - INTERVAL '30 days'
)
SELECT
    product_id,
    total_price,
    ROW_NUMBER() OVER (PARTITION BY product_id ORDER BY total_price DESC) AS rank
FROM recent_orders;

-- Subquery, EXISTS, and set operation (UNION)
SELECT name
FROM inventory.products
WHERE EXISTS (
    SELECT 1 FROM inventory.orders WHERE inventory.orders.product_id = inventory.products.id
)
UNION
SELECT name
FROM inventory.products
WHERE category IS NULL;

-- CAST and EXTRACT
SELECT
    CAST(price AS INTEGER) AS rounded_price,
    EXTRACT(YEAR FROM ordered_at) AS order_year
FROM inventory.orders o
JOIN inventory.products ON inventory.products.id = o.product_id;

INSERT INTO inventory.products (name, price, category)
VALUES ('Widget', 9.99, 'hardware');

UPDATE inventory.products
SET price = price * 1.1
WHERE category = 'hardware';

DELETE FROM inventory.orders
WHERE ordered_at < NOW() - INTERVAL '1 year';

-- MERGE with WHEN MATCHED / WHEN NOT MATCHED branches
MERGE INTO inventory.products AS t
USING inventory.orders AS s
ON t.id = s.product_id
WHEN MATCHED THEN
    UPDATE SET stock = t.stock - s.quantity
WHEN NOT MATCHED THEN
    INSERT (id, name, price, stock) VALUES (s.product_id, 'unknown', 0, 0);
