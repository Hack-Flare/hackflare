ALTER TABLE dns_records DROP CONSTRAINT dns_records_zone_id_name_rtype_data_key;
ALTER TABLE dns_records
    ADD CONSTRAINT dns_records_zone_id_name_rtype_key
    UNIQUE (zone_id, name, rtype);