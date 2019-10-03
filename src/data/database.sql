
CREATE TABLE anime
(
    anime_id SERIAL PRIMARY KEY,
    anime_name VARCHAR(100) NOT NULL
);

CREATE TABLE people
(
    character_id SERIAL UNIQUE PRIMARY KEY,
    anime_id SERIAL REFERENCES anime(anime_id),
    character_name VARCHAR(100) NOT NULL
);

CREATE TABLE users 
(
    user_id SERIAL PRIMARY KEY,
    username VARCHAR(30) NOT NULL
);

CREATE TABLE files
(
    file_id SERIAL PRIMARY KEY,
    link VARCHAR(200) NOT NULL,
    character_id SERIAL REFERENCES people(character_id),
    file_name VARCHAR(70) NOT NULL,
    unique(file_id, character_id)
);

CREATE TABLE rating 
(
    user_id SERIAL REFERENCES users(user_id),
    file_id SERIAL REFERENCES files(file_id),
    score SMALLINT NOT NULL
);


-- 
-- 
-- with curr_id as (
--     SELECT user_id FROM users WHERE username = 'TEST USERNAME'
-- ),
-- previous_ids as (
--     SELECT file_id FROM rating WHERE user_id = (SELECT * FROM curr_id)
-- ),
-- file_to_upload as (
--     SELECT previous_ids.file_id FROM previous_ids
--     LEFT JOIN files on files.character_id = previous_ids.file_id 
--     WHERE previous_ids.file_id is null
--     limit 1
-- ),
-- good_file_id as (
--     SELECT files.file_name, files.character_id FROM files where (select * from file_to_upload) = files.file_id
-- ),
-- character_name as (
--     SELECT character_name, anime_id from people where character_id = (select character_id from good_file_id)
-- ),
-- anime_name as (
--     select anime_name from anime where anime_id = (select anime_id from character_name)
-- )

-- select (select * from anime_name), (select character_name from character_name), (select file_name from good_file_id);