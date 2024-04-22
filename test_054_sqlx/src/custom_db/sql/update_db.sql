ALTER TABLE distributors 
    ADD COLUMN address varchar(20),
    DROP COLUMN address RESTRICT, -- Удаление ограничено
    ALTER COLUMN address TYPE varchar(50),
    ALTER COLUMN name TYPE varchar(50)