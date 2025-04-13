ALTER TABLE ingredients
ADD COLUMN mathem_category_name TEXT;

UPDATE ingredients
SET mathem_category_name = '78-mejeri-ost-juice/129-agg-jast/130-agg'
WHERE name = 'Ägg';

UPDATE ingredients
SET mathem_category_name = '329-skafferi/354-mjol-bakning/356-vetemjol'
WHERE name = 'Vetemjöl';

UPDATE ingredients
SET mathem_category_name = '329-skafferi/380-sylt-socker/382-socker'
WHERE name = 'Strösocker';

UPDATE ingredients
SET mathem_category_name = '329-skafferi/354-mjol-bakning/363-kakao-choklad'
WHERE name = 'Kakao';

UPDATE ingredients
SET mathem_category_name = '78-mejeri-ost-juice/113-smor-margarin/115-mat-bak-smormargarin'
WHERE name = 'Smör';

ALTER TABLE ingredients 
ALTER COLUMN mathem_category_name SET NOT NULL;
