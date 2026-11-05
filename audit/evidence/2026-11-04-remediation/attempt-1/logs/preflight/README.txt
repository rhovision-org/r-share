Preflight evidence copied from /tmp on 2026-11-04.

targeted1.log and targeted2.log both ran on the previous scratch chain on
2026-11-04 around 22:55 (file mtimes 22:55 and 22:56). They did NOT run on
attempt-1.

Each log contains one full pass of pause,admin,admin_keys,pause_scope; the two
logs together are two consecutive passes of that suite set:

  targeted1.log: 88503eae7ef5f505f602ee9bd63e1ee4b33b38db16f97738d1ebf2b06a1a7028
  targeted2.log: 883930c862d69f8cbf2db2d8cce85a7117be84b67482841e67a1d1085387eb20

The only run-to-run difference is the generated booking domain id/timestamp in
the admin suite output. The originals remain at /tmp/targeted1.log and
/tmp/targeted2.log.
