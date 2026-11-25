"""Use the IDEAM catalog location of the stations whose source location was rounded."""

from django.db import migrations

from apps.climate.models import CATALOG_LOCATIONS, station_location


def fix_locations(apps, schema_editor):
    Station = apps.get_model("climate", "Station")
    for code in CATALOG_LOCATIONS:
        Station.objects.filter(code=code).update(
            location=station_location(code, None, None)
        )


class Migration(migrations.Migration):
    dependencies = [("climate", "0001_initial")]

    operations = [migrations.RunPython(fix_locations, migrations.RunPython.noop)]
