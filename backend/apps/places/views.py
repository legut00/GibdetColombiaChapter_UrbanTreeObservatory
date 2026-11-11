from django_filters.rest_framework import DjangoFilterBackend
from rest_framework import filters, viewsets
from rest_framework.decorators import action
from rest_framework.permissions import IsAuthenticatedOrReadOnly
from rest_framework.response import Response

from .models import Country, Department, Locality, Municipality, Neighborhood, Site
from .serializers import (
    CountryGeoSerializer,
    CountrySerializer,
    DepartmentGeoSerializer,
    DepartmentSerializer,
    LocalityGeoSerializer,
    LocalitySerializer,
    MunicipalityGeoSerializer,
    MunicipalitySerializer,
    NeighborhoodGeoSerializer,
    NeighborhoodSerializer,
    SiteSerializer,
)
from .statistics import statistics_response


class CountryViewSet(viewsets.ReadOnlyModelViewSet):
    """API endpoint for Country model."""

    queryset = Country.objects.all()
    serializer_class = CountrySerializer
    permission_classes = [IsAuthenticatedOrReadOnly]
    filter_backends = [filters.SearchFilter, filters.OrderingFilter]
    search_fields = ["name"]
    ordering_fields = ["name"]
    ordering = ["name"]

    def get_serializer_class(self):
        """Return appropriate serializer class."""
        format_param = self.request.query_params.get("format")
        if format_param == "geojson":
            return CountryGeoSerializer
        return CountrySerializer


class DepartmentViewSet(viewsets.ReadOnlyModelViewSet):
    """API endpoint for Department model."""

    queryset = Department.objects.all()
    serializer_class = DepartmentSerializer
    permission_classes = [IsAuthenticatedOrReadOnly]
    filter_backends = [
        filters.SearchFilter,
        filters.OrderingFilter,
        DjangoFilterBackend,
    ]
    search_fields = ["name"]
    filterset_fields = ["country"]
    ordering_fields = ["name", "country__name"]
    ordering = ["name"]

    def get_serializer_class(self):
        """Return appropriate serializer class."""
        format_param = self.request.query_params.get("format")
        if format_param == "geojson":
            return DepartmentGeoSerializer
        return DepartmentSerializer


class MunicipalityViewSet(viewsets.ReadOnlyModelViewSet):
    """API endpoint for Municipality model."""

    queryset = Municipality.objects.all()
    serializer_class = MunicipalitySerializer
    permission_classes = [IsAuthenticatedOrReadOnly]
    filter_backends = [
        filters.SearchFilter,
        filters.OrderingFilter,
        DjangoFilterBackend,
    ]
    search_fields = ["name"]
    filterset_fields = {
        "department": ["exact"],
        "department__country": ["exact"],
    }
    ordering_fields = ["name", "department__name"]
    ordering = ["name"]

    def get_serializer_class(self):
        """Return appropriate serializer class."""
        format_param = self.request.query_params.get("format")
        if format_param == "geojson":
            return MunicipalityGeoSerializer
        return MunicipalitySerializer


class LocalityViewSet(viewsets.ReadOnlyModelViewSet):
    """API endpoint for Locality model."""

    queryset = Locality.objects.select_related(
        "municipality", "municipality__department", "municipality__department__country"
    )
    serializer_class = LocalitySerializer
    permission_classes = [IsAuthenticatedOrReadOnly]
    filter_backends = [
        filters.SearchFilter,
        filters.OrderingFilter,
        DjangoFilterBackend,
    ]
    search_fields = ["name"]
    filterset_fields = {
        "municipality": ["exact"],
        "municipality__department": ["exact"],
        "municipality__department__country": ["exact"],
    }
    ordering_fields = [
        "name",
        "municipality__name",
        "municipality__department__name",
        "calculated_area_m2",
        "population_2019",
    ]
    ordering = ["name"]

    def get_serializer_class(self):
        """Return appropriate serializer class."""
        format_param = self.request.query_params.get("format")
        if format_param == "geojson":
            return LocalityGeoSerializer
        return LocalitySerializer

    @action(detail=False, methods=["get"])
    def statistics(self, request):
        """Tree count, distinct species and average height for every locality (one
        entry per locality, filters apply), plus city-wide totals under ``overall``."""
        ids = self.filter_queryset(self.get_queryset()).values_list("id", flat=True)
        return Response(statistics_response(list(ids), "neighborhood__locality_id"))


class NeighborhoodViewSet(viewsets.ReadOnlyModelViewSet):
    """API endpoint for Neighborhood model."""

    queryset = Neighborhood.objects.select_related(
        "locality", "locality__municipality", "locality__municipality__department"
    )
    serializer_class = NeighborhoodSerializer
    permission_classes = [IsAuthenticatedOrReadOnly]
    filter_backends = [
        filters.SearchFilter,
        filters.OrderingFilter,
        DjangoFilterBackend,
    ]
    search_fields = ["name"]
    filterset_fields = {
        "locality": ["exact"],
        "locality__municipality": ["exact"],
        "locality__municipality__department": ["exact"],
        "locality__municipality__department__country": ["exact"],
    }
    ordering_fields = ["name", "locality__name", "locality__municipality__name"]
    ordering = ["name"]

    def get_serializer_class(self):
        """Return appropriate serializer class."""
        format_param = self.request.query_params.get("format")
        if format_param == "geojson":
            return NeighborhoodGeoSerializer
        return NeighborhoodSerializer

    @action(detail=False, methods=["get"])
    def statistics(self, request):
        """Tree count, distinct species and average height for every neighborhood (one
        entry per neighborhood, filters apply), plus city-wide totals under ``overall``."""
        ids = self.filter_queryset(self.get_queryset()).values_list("id", flat=True)
        return Response(statistics_response(list(ids), "neighborhood_id"))


class SiteViewSet(viewsets.ReadOnlyModelViewSet):
    """API endpoint for Site model."""

    queryset = Site.objects.all()
    serializer_class = SiteSerializer
    permission_classes = [IsAuthenticatedOrReadOnly]
    filter_backends = [
        filters.SearchFilter,
        filters.OrderingFilter,
        DjangoFilterBackend,
    ]
    search_fields = ["name"]
    filterset_fields = {
        "zone": ["exact"],
        "subzone": ["exact"],
    }
    ordering_fields = [
        "name",
        "zone",
        "subzone",
    ]
    ordering = ["name"]
