#!/bin/bash
set -e

psql -v ON_ERROR_STOP=1 --username "pgactix" --password "pgactix" <<-EOSQL
--
-- PostgreSQL database dump
--

-- Dumped from database version 11beta1 (Debian 11~beta1-2.pgdg90+1)
-- Dumped by pg_dump version 11.5

-- Started on 2019-10-01 19:17:03 CEST

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- TOC entry 7 (class 2615 OID 16409)
-- Name: intrared; Type: SCHEMA; Schema: -; Owner: pgactix
--

CREATE SCHEMA intrared;


ALTER SCHEMA intrared OWNER TO pgactix;

SET default_tablespace = '';

SET default_with_oids = false;

--
-- TOC entry 201 (class 1259 OID 16450)
-- Name: groups; Type: TABLE; Schema: intrared; Owner: pgactix
--

CREATE TABLE intrared.groups (
    id uuid NOT NULL,
    data jsonb,
    version integer NOT NULL
);


ALTER TABLE intrared.groups OWNER TO pgactix;

--
-- TOC entry 202 (class 1259 OID 16510)
-- Name: r_role_group; Type: TABLE; Schema: intrared; Owner: pgactix
--

CREATE TABLE intrared.r_role_group (
    id_role uuid NOT NULL,
    id_group uuid NOT NULL
);


ALTER TABLE intrared.r_role_group OWNER TO pgactix;

--
-- TOC entry 203 (class 1259 OID 16527)
-- Name: r_user_group; Type: TABLE; Schema: intrared; Owner: pgactix
--

CREATE TABLE intrared.r_user_group (
    id_user uuid NOT NULL,
    id_group uuid NOT NULL
);


ALTER TABLE intrared.r_user_group OWNER TO pgactix;

--
-- TOC entry 200 (class 1259 OID 16422)
-- Name: roles; Type: TABLE; Schema: intrared; Owner: pgactix
--

CREATE TABLE intrared.roles (
    id uuid NOT NULL,
    data jsonb,
    version integer NOT NULL
);


ALTER TABLE intrared.roles OWNER TO pgactix;

--
-- TOC entry 199 (class 1259 OID 16410)
-- Name: users; Type: TABLE; Schema: intrared; Owner: pgactix
--

CREATE TABLE intrared.users (
    id uuid NOT NULL,
    data jsonb,
    version integer NOT NULL
);


ALTER TABLE intrared.users OWNER TO pgactix;

--
-- TOC entry 2788 (class 2606 OID 16457)
-- Name: groups groups_pkey; Type: CONSTRAINT; Schema: intrared; Owner: pgactix
--

ALTER TABLE ONLY intrared.groups
    ADD CONSTRAINT groups_pkey PRIMARY KEY (id);


--
-- TOC entry 2792 (class 2606 OID 16514)
-- Name: r_role_group r_role_group_pkey; Type: CONSTRAINT; Schema: intrared; Owner: pgactix
--

ALTER TABLE ONLY intrared.r_role_group
    ADD CONSTRAINT r_role_group_pkey PRIMARY KEY (id_role, id_group);


--
-- TOC entry 2796 (class 2606 OID 16531)
-- Name: r_user_group r_user_group_pkey; Type: CONSTRAINT; Schema: intrared; Owner: pgactix
--

ALTER TABLE ONLY intrared.r_user_group
    ADD CONSTRAINT r_user_group_pkey PRIMARY KEY (id_user, id_group);


--
-- TOC entry 2783 (class 2606 OID 16429)
-- Name: roles roles_pkey; Type: CONSTRAINT; Schema: intrared; Owner: pgactix
--

ALTER TABLE ONLY intrared.roles
    ADD CONSTRAINT roles_pkey PRIMARY KEY (id);


--
-- TOC entry 2778 (class 2606 OID 16417)
-- Name: users users_pkey; Type: CONSTRAINT; Schema: intrared; Owner: pgactix
--

ALTER TABLE ONLY intrared.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- TOC entry 2789 (class 1259 OID 16525)
-- Name: fki_rg_id_group_fk; Type: INDEX; Schema: intrared; Owner: pgactix
--

CREATE INDEX fki_rg_id_group_fk ON intrared.r_role_group USING btree (id_group);


--
-- TOC entry 2790 (class 1259 OID 16526)
-- Name: fki_rg_id_role_fk; Type: INDEX; Schema: intrared; Owner: pgactix
--

CREATE INDEX fki_rg_id_role_fk ON intrared.r_role_group USING btree (id_role);


--
-- TOC entry 2793 (class 1259 OID 16542)
-- Name: fki_ug_id_group_fk; Type: INDEX; Schema: intrared; Owner: pgactix
--

CREATE INDEX fki_ug_id_group_fk ON intrared.r_user_group USING btree (id_group);


--
-- TOC entry 2794 (class 1259 OID 16543)
-- Name: fki_ug_id_user_fk; Type: INDEX; Schema: intrared; Owner: pgactix
--

CREATE INDEX fki_ug_id_user_fk ON intrared.r_user_group USING btree (id_user);


--
-- TOC entry 2784 (class 1259 OID 16458)
-- Name: group_data_ix; Type: INDEX; Schema: intrared; Owner: pgactix
--

CREATE INDEX group_data_ix ON intrared.groups USING gin (data);


--
-- TOC entry 2785 (class 1259 OID 16459)
-- Name: group_id_ix; Type: INDEX; Schema: intrared; Owner: pgactix
--

CREATE UNIQUE INDEX group_id_ix ON intrared.groups USING btree (id);


--
-- TOC entry 2786 (class 1259 OID 16460)
-- Name: group_name_ix; Type: INDEX; Schema: intrared; Owner: pgactix
--

CREATE UNIQUE INDEX group_name_ix ON intrared.groups USING btree (((data -> 'name'::text)));


--
-- TOC entry 2779 (class 1259 OID 16461)
-- Name: role_app_name_ix; Type: INDEX; Schema: intrared; Owner: pgactix
--

CREATE UNIQUE INDEX role_app_name_ix ON intrared.roles USING btree (((data -> 'app'::text)), ((data -> 'name'::text)));


--
-- TOC entry 2780 (class 1259 OID 16430)
-- Name: role_data_ix; Type: INDEX; Schema: intrared; Owner: pgactix
--

CREATE INDEX role_data_ix ON intrared.roles USING gin (data);


--
-- TOC entry 2781 (class 1259 OID 16432)
-- Name: role_id_ix; Type: INDEX; Schema: intrared; Owner: pgactix
--

CREATE UNIQUE INDEX role_id_ix ON intrared.roles USING btree (id);


--
-- TOC entry 2773 (class 1259 OID 16418)
-- Name: user_data_ix; Type: INDEX; Schema: intrared; Owner: pgactix
--

CREATE INDEX user_data_ix ON intrared.users USING gin (data);


--
-- TOC entry 2774 (class 1259 OID 16419)
-- Name: user_email_ix; Type: INDEX; Schema: intrared; Owner: pgactix
--

CREATE UNIQUE INDEX user_email_ix ON intrared.users USING btree (((data -> 'email'::text)));


--
-- TOC entry 2775 (class 1259 OID 16420)
-- Name: user_id_ix; Type: INDEX; Schema: intrared; Owner: pgactix
--

CREATE UNIQUE INDEX user_id_ix ON intrared.users USING btree (id);


--
-- TOC entry 2776 (class 1259 OID 16421)
-- Name: user_username_ix; Type: INDEX; Schema: intrared; Owner: pgactix
--

CREATE UNIQUE INDEX user_username_ix ON intrared.users USING btree (((data -> 'user_name'::text)));


--
-- TOC entry 2797 (class 2606 OID 16515)
-- Name: r_role_group id_group_fk; Type: FK CONSTRAINT; Schema: intrared; Owner: pgactix
--

ALTER TABLE ONLY intrared.r_role_group
    ADD CONSTRAINT id_group_fk FOREIGN KEY (id_group) REFERENCES intrared.groups(id) ON DELETE CASCADE;


--
-- TOC entry 2799 (class 2606 OID 16532)
-- Name: r_user_group id_group_fk; Type: FK CONSTRAINT; Schema: intrared; Owner: pgactix
--

ALTER TABLE ONLY intrared.r_user_group
    ADD CONSTRAINT id_group_fk FOREIGN KEY (id_group) REFERENCES intrared.groups(id) ON DELETE CASCADE;


--
-- TOC entry 2798 (class 2606 OID 16520)
-- Name: r_role_group id_role_fk; Type: FK CONSTRAINT; Schema: intrared; Owner: pgactix
--

ALTER TABLE ONLY intrared.r_role_group
    ADD CONSTRAINT id_role_fk FOREIGN KEY (id_role) REFERENCES intrared.roles(id) ON DELETE CASCADE;


--
-- TOC entry 2800 (class 2606 OID 16537)
-- Name: r_user_group id_user_fk; Type: FK CONSTRAINT; Schema: intrared; Owner: pgactix
--

ALTER TABLE ONLY intrared.r_user_group
    ADD CONSTRAINT id_user_fk FOREIGN KEY (id_user) REFERENCES intrared.users(id) ON DELETE CASCADE;


-- Completed on 2019-10-01 19:17:03 CEST

--
-- PostgreSQL database dump complete
--

EOSQL
