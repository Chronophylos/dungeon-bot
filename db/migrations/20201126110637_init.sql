-- Database generated with pgModeler (PostgreSQL Database Modeler).
-- pgModeler  version: 0.9.3-beta1
-- PostgreSQL version: 13.0
-- Project Site: pgmodeler.io
-- Model Author: ---

-- Database creation must be performed outside a multi lined SQL file. 
-- These commands were put in this file only as a convenience.
-- 
-- object: dungeon | type: DATABASE --
-- DROP DATABASE IF EXISTS dungeon;
-- CREATE DATABASE dungeon;
-- ddl-end --


-- object: public.race | type: TYPE --
-- DROP TYPE IF EXISTS public.race CASCADE;
CREATE TYPE public.race AS
 ENUM ('human');
-- ddl-end --

-- object: public.class | type: TYPE --
-- DROP TYPE IF EXISTS public.class CASCADE;
CREATE TYPE public.class AS
 ENUM ('fighter');
-- ddl-end --

-- object: public.player | type: TABLE --
-- DROP TABLE IF EXISTS public.player CASCADE;
CREATE TABLE public.player (
	id integer NOT NULL,
	dungeon_cooldown timestamptz,
	race public.race,
	class public.class,
	strength smallint,
	dexterity smallint,
	constitution smallint,
	intelligence smallint,
	wisdom smallint,
	charisma smallint,
	luck smallint,
	has_character bool NOT NULL DEFAULT false,
	CONSTRAINT player_pk PRIMARY KEY (id)

);
-- ddl-end --
COMMENT ON COLUMN public.player.id IS E'equals twitch user id';
-- ddl-end --
COMMENT ON COLUMN public.player.dungeon_cooldown IS E'When the dungeon cooldown is reset';
-- ddl-end --


