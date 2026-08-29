-- Allow multiple records with the same name + type (e.g. two A records, or multiple
-- TXT records on _acme-challenge for ACME dns01 verification of wildcard + root certs).
-- Exact-duplicate rows (same zone, name, type, data) are still prevented.
ALTER TABLE dns_records DROP CONSTRAINT dns_records_zone_id_name_rtype_key;
ALTER TABLE dns_records
    ADD CONSTRAINT dns_records_zone_id_name_rtype_data_key
    UNIQUE (zone_id, name, rtype, data);