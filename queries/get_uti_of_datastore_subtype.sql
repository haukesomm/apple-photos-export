SELECT asset.Z_PK          AS ID,
       int_res.ZCOMPACTUTI AS COMPACT_UTI
FROM ZASSET asset
         INNER JOIN ZADDITIONALASSETATTRIBUTES asset_attribs
                    ON asset.Z_PK = asset_attribs.ZASSET
         LEFT JOIN ZINTERNALRESOURCE int_res
                   ON int_res.ZASSET = asset_attribs.ZASSET
WHERE asset.Z_PK = ?
  AND int_res.ZDATASTORESUBTYPE = ?