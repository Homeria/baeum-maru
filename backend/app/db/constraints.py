CAPACITY_CHECK = """
(
  capacity_type = 'fixed'
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
  AND male_capacity >= 0
  AND female_capacity >= 0
  AND male_capacity + female_capacity > 0
)
"""
