SELECT asset.Z_PK                           AS ID,
       asset.ZUUID                          AS UUID,
       asset.ZDIRECTORY                     AS DIR,
       asset.ZFILENAME                      AS FILENAME,
       asset.ZUNIFORMTYPEIDENTIFIER         AS UTI,
       asset.ZDATECREATED                   AS DATETIME,
       asset.ZHIDDEN                        AS HIDDEN,
       asset.ZTRASHEDSTATE                  AS TRASHED,
       asset.ZVISIBILITYSTATE               AS VISIBLE,
       asset.ZDUPLICATEASSETVISIBILITYSTATE AS DUPLICATE_VISIBILITY,
       asset.ZADJUSTMENTSSTATE > 0          AS HAS_ADJUSTMENTS,
       asset_attribs.ZORIGINALFILENAME      AS ORIGINAL_FILENAME,
       int_res.ZCOMPACTUTI                  AS COMPACT_UTI,
       GROUP_CONCAT(album.Z_PK, ', ')       AS ALBUM_IDS
FROM ZASSET asset
         INNER JOIN ZADDITIONALASSETATTRIBUTES asset_attribs
                    ON asset.Z_PK = asset_attribs.ZASSET
         LEFT JOIN ZINTERNALRESOURCE int_res
                   ON int_res.ZASSET = asset_attribs.ZASSET
                       AND int_res.ZDATASTORESUBTYPE = 1
         LEFT JOIN Z__ALBUM_Z_ENT_ASSETS album_mapping
                   ON album_mapping.Z__ASSET_Z_ENT_ASSETS = asset.Z_PK
         LEFT JOIN ZGENERICALBUM album
                   ON album_mapping.Z__ALBUM_Z_ENT_ALBUMS = album.Z_PK
WHERE asset.ZTRASHEDSTATE = false
  AND asset.ZVISIBILITYSTATE = 0
  AND asset.ZDUPLICATEASSETVISIBILITYSTATE = 0
  -- Field may not be filled in the database, depending on whether it is an iCloud-enabled or
  -- offline library
  AND (int_res.ZLOCALAVAILABILITY = 1 OR int_res.ZLOCALAVAILABILITY IS NULL)
  -- Album kind values:
  -- - 3999: Root album
  -- - 4000: User-created folder
  -- - 2: User-created album
  AND (album.ZKIND IS NULL OR (album.ZTRASHEDSTATE = false AND album.ZKIND IN (3999, 4000, 2)))
GROUP BY asset.Z_PK
ORDER BY asset.Z_PK