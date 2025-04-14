SELECT count(*)
FROM ZASSET asset
         INNER JOIN ZADDITIONALASSETATTRIBUTES asset_attribs
                    ON asset.Z_PK = asset_attribs.ZASSET
         LEFT OUTER JOIN ZINTERNALRESOURCE int_res
                         ON (asset_attribs.ZASSET = int_res.ZASSET)
                             AND int_res.ZDATASTORESUBTYPE = 1
WHERE asset.ZTRASHEDSTATE = false
  AND asset.ZVISIBILITYSTATE = 0
  AND asset.ZDUPLICATEASSETVISIBILITYSTATE = 0;