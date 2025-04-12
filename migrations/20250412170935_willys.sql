ALTER TABLE ingredients
ADD COLUMN willys_category_name TEXT;

UPDATE ingredients
SET willys_category_name = 'mejeri-ost-och-agg/agg'
WHERE name = 'Ägg';

UPDATE ingredients
SET willys_category_name = 'skafferi/bakning/mjol'
WHERE name = 'Vetemjöl';

UPDATE ingredients
SET willys_category_name = 'skafferi/bakning/socker-och-honung'
WHERE name = 'Strösocker';

UPDATE ingredients
SET willys_category_name = 'skafferi/bakning/baktillbehor'
WHERE name = 'Kakao';

UPDATE ingredients
SET willys_category_name = 'mejeri-ost-och-agg/smor-margarin-och-jast/matfett'
WHERE name = 'Smör';

ALTER TABLE ingredients 
ALTER COLUMN willys_category_name SET NOT NULL;
