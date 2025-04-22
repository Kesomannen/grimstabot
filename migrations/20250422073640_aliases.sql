ALTER TABLE ingredients
ADD COLUMN aliases TEXT NOT NULL DEFAULT '';

UPDATE ingredients
SET aliases = 'Kakaopulver,Ögoncacao'
WHERE name = 'Kakao';
