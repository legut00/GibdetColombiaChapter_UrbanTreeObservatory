import pytest

from apps.climate.models import station_location


@pytest.mark.django_db
def test_station_str(station):
    """Test the string representation of the station."""
    assert str(station) == f"{station.name} ({station.code})"


@pytest.mark.django_db
def test_climate_str(climate_record):
    """Test the string representation of the climate record."""
    assert (
        str(climate_record)
        == f"{climate_record.station} - {climate_record.date} - {climate_record.value} {climate_record.measure_unit}"
    )


def test_station_location_uses_the_ideam_catalog():
    perales = station_location(80214, -75.15, 4.4333)
    assert (perales.x, perales.y) == (-75.13941667, 4.424138889)
    other = station_location(1, -75.2, 4.4)
    assert (other.x, other.y) == (-75.2, 4.4)
