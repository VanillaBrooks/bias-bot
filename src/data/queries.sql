
-- insert a new file into the database

with curr_anime_id as (
    select anime_id from anime where anime_name = $1
),
curr_char_id as (
    select character_id from people WHERE character_name = $2 AND anime_id = (select * from curr_anime_id)
)
INSERT INTO files (link, character_id, file_name) VALUES ($3, (select * from curr_char_id), $4)



-- 
-- fetch a picture to show a user
-- 

with curr_id as (
    SELECT user_id FROM users WHERE username = 'TEST USERNAME'
),
previous_ids as (
    SELECT file_id FROM rating WHERE user_id = (SELECT * FROM curr_id)
),
file_to_upload as (
    SELECT previous_ids.file_id FROM previous_ids
    LEFT JOIN files on files.character_id = previous_ids.file_id 
    WHERE previous_ids.file_id is null
    limit 1
),
good_file_id as (
    SELECT files.file_name, files.character_id, files.file_id FROM files where (select * from file_to_upload) = files.file_id
),
character_name as (
    SELECT character_name, anime_id from people where character_id = (select character_id from good_file_id)
),
anime_name as (
    select anime_name from anime where anime_id = (select anime_id from character_name)
)

select (select * from anime_name), (select character_name from character_name), (select file_name from good_file_id), (select file_id from files), (select user_id from curr_id);




-- insert a new rating into the database

with curr_user_id as (
    SELECT user_id FROM users WHERE username = "TEST USER"
),
INSERT INTO rating (user_id, file_id, score) VALUES (1, 1, 1);

-- create a new user 

INSERT INTO users (username) VALUES ($1)

-- create a new anime

INSERT INTO anime (anime_name) VALUES ($1)

-- create new character
-- $1 = name of anime
-- $2 = character name

with anime_id as (
    SELECT anime_id FROM anime WHERE anime_name = $1
)
INSERT INTO people (anime_id, character_name) VALUES ( (SELECT * FROM anime_id), $2)

find if a link already exists

SELECT link FROM files WHERE link = $1