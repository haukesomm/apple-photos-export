SELECT a.Z_PK
     , a.ZTITLE
     , a.ZPARENTFOLDER
     , a.ZSTARTDATE
FROM ZGENERICALBUM a
-- Album kind values:
-- - 3999: Root album
-- - 4000: User-created folder
-- - 2: User-created album
WHERE a.ZKIND IN (3999, 4000, 2)
  AND a.ZTRASHEDSTATE = FALSE
ORDER BY a.ZSTARTDATE; 