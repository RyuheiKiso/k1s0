-- べき等性ガード: カラム追加が重複実行されても安全に処理する
ALTER TABLE policy.policies ADD COLUMN IF NOT EXISTS bundle_id VARCHAR(255);
