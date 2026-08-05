-- hickory's ResponseCode::to_string() produced human-readable phrases
-- ("No Error", "Server Failure", ...) instead of RFC mnemonics ("NOERROR",
-- "SERVFAIL", ...). Backfill any rows that were stored with the old format.
UPDATE dns_query_logs SET response_code = 'NOERROR' WHERE response_code = 'No Error';
UPDATE dns_query_logs SET response_code = 'FORMERR' WHERE response_code = 'Form Error';
UPDATE dns_query_logs SET response_code = 'SERVFAIL' WHERE response_code = 'Server Failure';
UPDATE dns_query_logs SET response_code = 'NXDOMAIN' WHERE response_code = 'Non-Existent Domain';
UPDATE dns_query_logs SET response_code = 'NOTIMP' WHERE response_code = 'Not Implemented';
UPDATE dns_query_logs SET response_code = 'REFUSED' WHERE response_code = 'Query Refused';
UPDATE dns_query_logs SET response_code = 'YXDOMAIN' WHERE response_code = 'Name should not exist';
UPDATE dns_query_logs SET response_code = 'XRRSET' WHERE response_code = 'RR Set should not exist';
UPDATE dns_query_logs SET response_code = 'NXRRSET' WHERE response_code = 'RR Set does not exist';
UPDATE dns_query_logs SET response_code = 'NOTAUTH' WHERE response_code = 'Not authorized';
UPDATE dns_query_logs SET response_code = 'NOTZONE' WHERE response_code = 'Name not in zone';
UPDATE dns_query_logs SET response_code = 'BADVERS' WHERE response_code = 'Bad option versions';
UPDATE dns_query_logs SET response_code = 'BADSIG' WHERE response_code = 'TSIG Failure';
UPDATE dns_query_logs SET response_code = 'BADKEY' WHERE response_code = 'Key not recognized';
UPDATE dns_query_logs SET response_code = 'BADTIME' WHERE response_code = 'Signature out of time window';
UPDATE dns_query_logs SET response_code = 'BADMODE' WHERE response_code = 'Bad TKEY mode';
UPDATE dns_query_logs SET response_code = 'BADNAME' WHERE response_code = 'Duplicate key name';
UPDATE dns_query_logs SET response_code = 'BADALG' WHERE response_code = 'Algorithm not supported';
UPDATE dns_query_logs SET response_code = 'BADTRUNC' WHERE response_code = 'Bad truncation';
UPDATE dns_query_logs SET response_code = 'BADCOOKIE' WHERE response_code = 'Bad/missing Server Cookie';
