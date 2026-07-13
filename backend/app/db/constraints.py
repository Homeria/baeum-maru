"""여러 SQLAlchemy 모델이 공유하는 DB CHECK 표현식."""

CAPACITY_CHECK = """
(
  capacity_type = 'fixed'
  AND capacity_total IS NOT NULL
  AND capacity_total > 0
  AND male_capacity IS NULL
  AND female_capacity IS NULL
)
OR (
  capacity_type = 'open'
  AND capacity_total IS NULL
  AND male_capacity IS NULL
  AND female_capacity IS NULL
)
OR (
  capacity_type = 'gender_split'
  AND capacity_total IS NULL
  AND male_capacity IS NOT NULL
  AND female_capacity IS NOT NULL
  AND male_capacity >= 0
  AND female_capacity >= 0
  AND male_capacity + female_capacity > 0
)
"""
