# Test jiff datetime integration (feature = "jiff")

# region: Timestamp (POSIXct UTC)

test_that("jiff: Timestamp roundtrips through POSIXct (UTC)", {
  ts <- as.POSIXct("2024-01-15 12:30:00", tz = "UTC")
  result <- jiff_roundtrip_timestamp(ts)
  expect_s3_class(result, "POSIXct")
  expect_equal(as.numeric(result), as.numeric(ts), tolerance = 1e-6)
})

test_that("jiff: Timestamp roundtrips epoch correctly", {
  epoch <- jiff_epoch_timestamp()
  expect_s3_class(epoch, "POSIXct")
  expect_equal(as.numeric(epoch), 0.0)
})

test_that("jiff: negative Timestamp (pre-1970) roundtrips correctly", {
  neg <- jiff_negative_timestamp()
  expect_s3_class(neg, "POSIXct")
  expect_true(as.numeric(neg) < 0)
})

test_that("jiff: fractional-second Timestamp roundtrips with subsecond precision", {
  frac <- jiff_fractional_timestamp()
  expect_s3_class(frac, "POSIXct")
  # Should be 1.5 seconds after epoch
  expect_equal(as.numeric(frac), 1.5, tolerance = 1e-6)
})

test_that("jiff: -0.5 second Timestamp roundtrips correctly (floor-based split)", {
  half_before <- jiff_half_second_before_epoch()
  expect_s3_class(half_before, "POSIXct")
  # Should be -0.5 seconds (floor(-0.5) = -1, subsec = 0.5)
  expect_equal(as.numeric(half_before), -0.5, tolerance = 1e-6)
})

test_that("jiff: Vec<Timestamp> roundtrips as POSIXct vector", {
  now <- Sys.time()
  ts_vec <- c(now, now + 3600, now - 3600)
  attr(ts_vec, "tzone") <- "UTC"
  result <- jiff_roundtrip_timestamp_vec(ts_vec)
  expect_s3_class(result, "POSIXct")
  expect_length(result, 3L)
  expect_equal(as.numeric(result), as.numeric(ts_vec), tolerance = 1e-6)
})

test_that("jiff: Option<Timestamp> maps NA to NA POSIXct and roundtrips Some", {
  # NULL / NA_real_ → None → NA POSIXct (not NULL)
  na_result <- jiff_option_timestamp(as.POSIXct(NA))
  expect_s3_class(na_result, "POSIXct")
  expect_true(is.na(as.numeric(na_result)))

  ts <- as.POSIXct("2024-06-01", tz = "UTC")
  some_result <- jiff_option_timestamp(ts)
  expect_s3_class(some_result, "POSIXct")
  expect_equal(as.numeric(some_result), as.numeric(ts), tolerance = 1e-6)
})

test_that("jiff: jiff_timestamp_seconds extracts correct value", {
  ts <- as.POSIXct("1970-01-01 00:00:01.5", tz = "UTC")
  secs <- jiff_timestamp_seconds(ts)
  expect_equal(secs, 1.5, tolerance = 1e-6)
})

# endregion

# region: Zoned (POSIXct + tzone)

test_that("jiff: Zoned roundtrips America/New_York timezone", {
  skip_if_not(exists("jiff_roundtrip_zoned"), "jiff_roundtrip_zoned not available")
  ts <- as.POSIXct("2024-06-15 10:00:00", tz = "America/New_York")
  result <- jiff_roundtrip_zoned(ts)
  expect_s3_class(result, "POSIXct")
  expect_equal(as.numeric(result), as.numeric(ts), tolerance = 1e-6)
  expect_equal(attr(result, "tzone"), "America/New_York")
})

test_that("jiff: Zoned roundtrips UTC", {
  ts <- as.POSIXct("2024-01-01 00:00:00", tz = "UTC")
  result <- jiff_roundtrip_zoned(ts)
  expect_s3_class(result, "POSIXct")
  expect_equal(as.numeric(result), as.numeric(ts), tolerance = 1e-6)
})

test_that("jiff: jiff_zoned_tz_name extracts timezone name", {
  ts <- as.POSIXct("2024-06-15 10:00:00", tz = "Europe/London")
  tz_name <- jiff_zoned_tz_name(ts)
  expect_equal(tz_name, "Europe/London")
})

test_that("jiff: jiff_zoned_year extracts year", {
  ts <- as.POSIXct("2024-06-15 10:00:00", tz = "UTC")
  yr <- jiff_zoned_year(ts)
  expect_equal(yr, 2024L)
})

test_that("jiff: jiff_zoned_month extracts month (1-12)", {
  ts <- as.POSIXct("2024-06-15 10:00:00", tz = "UTC")
  expect_equal(jiff_zoned_month(ts), 6L)
})

test_that("jiff: Vec<Zoned> roundtrips as POSIXct vector", {
  ts1 <- as.POSIXct("2024-01-01 00:00:00", tz = "UTC")
  ts2 <- as.POSIXct("2024-06-15 12:00:00", tz = "UTC")
  ts_vec <- c(ts1, ts2)
  result <- jiff_roundtrip_zoned_vec(ts_vec)
  expect_s3_class(result, "POSIXct")
  expect_length(result, 2L)
  expect_equal(as.numeric(result), as.numeric(ts_vec), tolerance = 1e-6)
})

test_that("jiff: unknown IANA timezone returns error (not silent UTC)", {
  bad <- structure(0, class = c("POSIXct", "POSIXt"), tzone = "Mars/Olympus")
  expect_error(jiff_roundtrip_zoned(bad), regexp = "Mars/Olympus")
})

test_that("jiff: Vec<Zoned> with mixed tzs uses first tz for vector attribute", {
  # Two elements in different timezones: the vector-level tzone attr should match the first element.
  ts_ny <- as.POSIXct("2024-01-01 12:00:00", tz = "America/New_York")
  ts_london <- as.POSIXct("2024-06-15 08:00:00", tz = "Europe/London")
  # R's c() will coerce to the same numeric base but the tzone is taken from the first element
  ts_vec <- c(ts_ny, ts_london)
  attr(ts_vec, "tzone") <- "America/New_York"  # first element's tz dominates
  result <- jiff_roundtrip_zoned_vec(ts_vec)
  expect_s3_class(result, "POSIXct")
  expect_length(result, 2L)
  # After roundtrip the vector carries the first element's tz (America/New_York)
  expect_equal(attr(result, "tzone"), "America/New_York")
})

# endregion

# region: civil::Date (R Date)

test_that("jiff: civil::Date roundtrips through R Date", {
  d <- as.Date("2024-03-20")
  result <- jiff_roundtrip_date(d)
  expect_s3_class(result, "Date")
  expect_equal(as.numeric(result), as.numeric(d))
})

test_that("jiff: civil::Date vector roundtrips correctly", {
  dates <- as.Date(c("2024-01-01", "2000-06-15", "1970-01-01"))
  result <- jiff_roundtrip_date_vec(dates)
  expect_s3_class(result, "Date")
  expect_equal(as.numeric(result), as.numeric(dates))
})

test_that("jiff: epoch Date (1970-01-01) roundtrips", {
  epoch <- jiff_epoch_date()
  expect_s3_class(epoch, "Date")
  expect_equal(as.numeric(epoch), 0.0)
})

test_that("jiff: distant past Date (1900-03-01) roundtrips", {
  past <- jiff_distant_past_date()
  expect_s3_class(past, "Date")
  expect_true(as.numeric(past) < -25000)
})

test_that("jiff: jiff_date_year/month/day extract components", {
  d <- as.Date("2024-07-04")
  expect_equal(jiff_date_year(d), 2024L)
  expect_equal(jiff_date_month(d), 7L)
  expect_equal(jiff_date_day(d), 4L)
})

test_that("jiff: leap-day civil::Date (2024-02-29) roundtrips", {
  leap <- as.Date("2024-02-29")
  result <- jiff_roundtrip_date(leap)
  expect_s3_class(result, "Date")
  expect_equal(result, leap)
  expect_equal(jiff_date_year(leap), 2024L)
  expect_equal(jiff_date_month(leap), 2L)
  expect_equal(jiff_date_day(leap), 29L)
})

# endregion

# region: SignedDuration (difftime)

test_that("jiff: SignedDuration roundtrips 0", {
  dur <- structure(0, class = "difftime", units = "secs")
  result <- jiff_roundtrip_duration(dur)
  expect_s3_class(result, "difftime")
  expect_equal(as.numeric(result, units = "secs"), 0.0, tolerance = 1e-9)
})

test_that("jiff: jiff_one_hour_duration returns 3600s difftime", {
  dur <- jiff_one_hour_duration()
  expect_s3_class(dur, "difftime")
  expect_equal(as.numeric(dur, units = "secs"), 3600.0, tolerance = 1e-9)
})

test_that("jiff: jiff_negative_duration returns -0.5s difftime", {
  dur <- jiff_negative_duration()
  expect_s3_class(dur, "difftime")
  expect_equal(as.numeric(dur, units = "secs"), -0.5, tolerance = 1e-9)
})

test_that("jiff: jiff_duration_secs extracts seconds correctly", {
  dur <- structure(42.75, class = "difftime", units = "secs")
  secs <- jiff_duration_secs(dur)
  expect_equal(secs, 42.75, tolerance = 1e-9)
})

# endregion

# region: ALTREP (JiffTimestampVec)

test_that("jiff: ALTREP JiffTimestampVec has correct length", {
  altrep <- jiff_altrep_timestamps(10L)
  expect_length(altrep, 10L)
  expect_equal(jiff_altrep_len(altrep), 10L)
})

test_that("jiff: ALTREP JiffTimestampVec elements are correct", {
  altrep <- jiff_altrep_timestamps(5L)
  # Elements should be 0, 1, 2, 3, 4 seconds after epoch
  for (i in 0:4) {
    elt <- jiff_altrep_elt(altrep, i)
    expect_equal(elt, as.numeric(i), tolerance = 1e-9)
  }
})

test_that("jiff: ALTREP JiffTimestampVec is POSIXct class", {
  altrep <- jiff_altrep_timestamps(3L)
  expect_s3_class(altrep, "POSIXct")
})

test_that("jiff: empty ALTREP JiffTimestampVec has length 0", {
  altrep <- jiff_altrep_timestamps(0L)
  expect_length(altrep, 0L)
})

# endregion

# region: ALTREP (JiffZonedVec)

test_that("jiff: JiffZonedVec constructor preserves values and timezone", {
  input <- c(
    "2025-01-01T00:00:00[America/New_York]",
    "2025-06-01T12:00:00[America/New_York]"
  )
  result <- jiff_zoned_vec_new(input)

  expect_s3_class(result, "POSIXct")
  expect_length(result, 2L)
  expect_identical(attr(result, "tzone"), "America/New_York")
  expect_identical(
    jiff_zoned_vec_first_element(result),
    "2025-01-01T00:00:00-05:00[America/New_York]"
  )
})

test_that("jiff: JiffZonedVec rejects mixed timezone input", {
  input <- c(
    "2025-01-01T00:00:00[America/New_York]",
    "2025-01-01T00:00:00[Europe/London]"
  )
  expect_error(jiff_zoned_vec_new(input), "timezone")
})

# endregion

# region: civil::DateTime ExternalPtr (RDateTime adapter trait)

test_that("jiff: civil::DateTime ExternalPtr component accessors work", {
  dt <- jiff_datetime_new(year = 2024L, month = 6L, day = 15L,
                          hour = 10L, minute = 30L, second = 45L)
  expect_equal(jiff_datetime_year(dt), 2024L)
  expect_equal(jiff_datetime_month(dt), 6L)
  expect_equal(jiff_datetime_day(dt), 15L)
  expect_equal(jiff_datetime_hour(dt), 10L)
})

# endregion

# region: civil::Time ExternalPtr (RTime adapter trait)

test_that("jiff: civil::Time ExternalPtr component accessors work", {
  t <- jiff_time_new(hour = 14L, minute = 30L, second = 59L)
  expect_equal(jiff_time_hour(t), 14L)
  expect_equal(jiff_time_minute(t), 30L)
  expect_equal(jiff_time_second(t), 59L)
})

# endregion

# region: Span ExternalPtr (RSpan adapter trait)

test_that("jiff: Span ExternalPtr component accessors work", {
  s <- jiff_span_new(years = 1L, months = 2L, days = 15L)
  expect_equal(jiff_span_years(s), 1L)
  expect_equal(jiff_span_months(s), 2L)
  expect_equal(jiff_span_days(s), 15L)
  expect_false(jiff_span_is_zero(s))
  expect_false(jiff_span_is_negative(s))
})

test_that("jiff: zero Span is_zero returns TRUE", {
  zero <- jiff_span_new(years = 0L, months = 0L, days = 0L)
  expect_true(jiff_span_is_zero(zero))
  expect_false(jiff_span_is_negative(zero))
})

test_that("jiff: negative Span is_negative returns TRUE", {
  neg <- jiff_span_new(years = -1L, months = 0L, days = 0L)
  expect_true(jiff_span_is_negative(neg))
  expect_false(jiff_span_is_zero(neg))
})

# endregion

# region: RDate calendar helpers

test_that("jiff: jiff_date_weekday returns correct ISO weekday", {
  # 2024-01-01 is a Monday (ISO 1)
  d <- as.Date("2024-01-01")
  expect_equal(jiff_date_weekday(d), 1L)
  # 2024-01-07 is a Sunday (ISO 7)
  expect_equal(jiff_date_weekday(as.Date("2024-01-07")), 7L)
})

test_that("jiff: jiff_date_day_of_year returns correct ordinal", {
  expect_equal(jiff_date_day_of_year(as.Date("2024-01-01")), 1L)
  expect_equal(jiff_date_day_of_year(as.Date("2024-12-31")), 366L)  # 2024 is a leap year
  expect_equal(jiff_date_day_of_year(as.Date("2023-12-31")), 365L)
})

test_that("jiff: jiff_date_first_of_month returns first day of month", {
  result <- jiff_date_first_of_month(as.Date("2024-03-15"))
  expect_s3_class(result, "Date")
  expect_equal(format(result), "2024-03-01")
})

test_that("jiff: jiff_date_last_of_month returns last day of month", {
  # Leap year February
  result <- jiff_date_last_of_month(as.Date("2024-02-05"))
  expect_s3_class(result, "Date")
  expect_equal(format(result), "2024-02-29")
  # Non-leap year February
  result2 <- jiff_date_last_of_month(as.Date("2023-02-05"))
  expect_equal(format(result2), "2023-02-28")
})

test_that("jiff: jiff_date_tomorrow advances by one day", {
  result <- jiff_date_tomorrow(as.Date("2024-01-31"))
  expect_s3_class(result, "Date")
  expect_equal(format(result), "2024-02-01")
})

test_that("jiff: jiff_date_yesterday retreats by one day", {
  result <- jiff_date_yesterday(as.Date("2024-03-01"))
  expect_s3_class(result, "Date")
  expect_equal(format(result), "2024-02-29")  # 2024 is a leap year
})

# endregion

# region: RZoned adapter trait — start_of_day + strftime

test_that("jiff: jiff_zoned_start_of_day resets time to midnight, preserves tz", {
  zdt <- as.POSIXct("2024-06-15 14:30:45", tz = "America/New_York")
  result <- jiff_zoned_start_of_day(zdt)
  expect_s3_class(result, "POSIXct")
  expect_equal(attr(result, "tzone"), "America/New_York")
  # Start of day should be midnight local time
  lt <- as.POSIXlt(result, tz = "America/New_York")
  expect_equal(lt$hour, 0L)
  expect_equal(lt$min,  0L)
  expect_equal(lt$sec,  0L)
})

test_that("jiff: jiff_zoned_strftime formats datetime correctly", {
  zdt <- as.POSIXct("2024-01-15 09:05:03", tz = "UTC")
  result <- jiff_zoned_strftime(zdt, "%Y-%m-%d")
  expect_equal(result, "2024-01-15")
  result2 <- jiff_zoned_strftime(zdt, "%H:%M:%S")
  expect_equal(result2, "09:05:03")
})

# endregion

# region: RTimestamp adapter trait — strftime + as_millisecond

test_that("jiff: jiff_timestamp_strftime formats UTC timestamp correctly", {
  ts <- as.POSIXct("2024-06-15 12:30:45", tz = "UTC")
  result <- jiff_timestamp_strftime(ts, "%Y-%m-%dT%H:%M:%SZ")
  expect_equal(result, "2024-06-15T12:30:45Z")
})

test_that("jiff: jiff_timestamp_as_millisecond returns correct ms since epoch", {
  # 1 second after epoch = 1000 ms
  ts <- as.POSIXct(1.0, origin = "1970-01-01", tz = "UTC")
  ms <- jiff_timestamp_as_millisecond(ts)
  expect_equal(ms, 1000.0)
  # Epoch itself = 0 ms
  expect_equal(jiff_timestamp_as_millisecond(as.POSIXct(0, origin = "1970-01-01", tz = "UTC")), 0.0)
  # Negative (pre-1970)
  neg_ts <- as.POSIXct(-1.0, origin = "1970-01-01", tz = "UTC")
  expect_equal(jiff_timestamp_as_millisecond(neg_ts), -1000.0)
})

# endregion

# region: ALTREP laziness (item 9)

test_that("jiff: JiffTimestampVecCounted elt counter starts at 0 after creation", {
  altrep <- jiff_counted_altrep(10L)
  expect_equal(jiff_counted_altrep_elt_count(), 0L)
})

test_that("jiff: JiffTimestampVecCounted counter increments only on element access", {
  altrep <- jiff_counted_altrep(5L)
  # No access yet — counter must still be 0 (proves no eager materialization)
  expect_equal(jiff_counted_altrep_elt_count(), 0L)
  # Access element 0 via R's [[ — triggers exactly one elt() call
  invisible(altrep[[1]])
  expect_equal(jiff_counted_altrep_elt_count(), 1L)
  # Access element 2 — counter must be 2
  invisible(altrep[[3]])
  expect_equal(jiff_counted_altrep_elt_count(), 2L)
})

test_that("jiff: JiffTimestampVecCounted does not materialise on length query", {
  altrep <- jiff_counted_altrep(100L)
  expect_equal(length(altrep), 100L)
  expect_equal(jiff_counted_altrep_elt_count(), 0L)
})

# endregion

# region: vctrs rcrd constructors

test_that("jiff: Span vctrs rcrd has correct fields", {
  if (!exists("jiff_span_rcrd_demo", mode = "function")) skip("vctrs feature off")
  if (!requireNamespace("vctrs", quietly = TRUE)) skip("vctrs not installed")
  v <- jiff_span_rcrd_demo()
  expect_s3_class(v, "jiff_span")
  expect_s3_class(v, "vctrs_rcrd")
  expect_s3_class(v, "vctrs_vctr")
  expect_equal(vctrs::vec_size(v), 3L)
  expect_equal(vctrs::field(v, "years"), c(1L, 0L, 0L))
  expect_equal(vctrs::field(v, "months"), c(2L, 3L, 0L))
  expect_equal(vctrs::field(v, "days"), c(0L, 15L, 0L))
})

test_that("jiff: Zoned vctrs rcrd preserves per-element IANA name", {
  if (!exists("jiff_zoned_rcrd_demo", mode = "function")) skip("vctrs feature off")
  if (!requireNamespace("vctrs", quietly = TRUE)) skip("vctrs not installed")
  v <- jiff_zoned_rcrd_demo()
  expect_s3_class(v, "jiff_zoned")
  expect_equal(vctrs::vec_size(v), 2L)
  expect_equal(vctrs::field(v, "tz"), c("UTC", "Europe/Paris"))
})

# endregion
