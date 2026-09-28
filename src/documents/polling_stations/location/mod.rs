use crate::{
    EMLErrorKind, EMLVersion, EMLVersionRange, NS_SB,
    io::{
        EMLElement, EMLElementReader, EMLParsingMode, OwnedQualifiedName, QualifiedName,
        collect_struct,
    },
    utils::{StringValue, XsDateTime},
};

mod accessibility;
mod bag_id;
mod building_usage;
mod district_code;
mod geographic_location;
mod neighbourhood_code;
mod postal_code;
mod rd_geographic_location;
mod website;

pub use accessibility::*;
pub use bag_id::*;
pub use building_usage::*;
pub use district_code::*;
pub use geographic_location::*;
pub use neighbourhood_code::*;
pub use postal_code::*;
pub use rd_geographic_location::*;
pub use website::*;

/// Location details of a polling station since EML 1.3
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Location {
    /// The BAG ID of the polling station location
    pub bag_id: Option<StringValue<LocationBagId>>,

    /// The name of the street (i.e. openbare ruimte) where the polling station is located
    pub street_name: Option<Box<str>>,

    /// The house number of the polling station location address
    pub number: Option<StringValue<i64>>,

    /// The house letter (in addition to the house number) of the polling station location address
    pub letter: Option<Box<str>>,

    /// The number addition (adding to the letter and house number) of the polling station location address
    pub number_addition: Option<Box<str>>,

    /// The postal code of polling station location address
    pub postal_code: Option<StringValue<LocationPostalCode>>,

    /// The name of the city/town where the polling station is located
    pub city: Option<Box<str>>,

    /// Any additional address information for the polling station location
    pub additional_address_information: Option<Box<str>>,

    /// The usage of the building where the polling station is located
    pub building_usage: Option<StringValue<BuildingUsage>>,

    /// The name of the district ('wijk') where the polling station is located.
    ///
    /// Note: this is different from the district as defined by the rest of the
    /// EML specification, but specifically relates to the BAG meaning of a
    /// district.
    pub district_name: Option<Box<str>>,

    /// The code of the district ('wijkcode') where the polling station is located.
    ///
    /// Note: this is different from the district as defined by the rest of the
    /// EML specification, but specifically relates to the BAG meaning of a
    /// district.
    pub district_code: Option<StringValue<LocationDistrictCode>>,

    /// The name of the neighbourhood ('buurt') where the polling station is located.
    pub neighbourhood_name: Option<Box<str>>,

    /// The code of the neighbourhood ('buurtcode') where the polling station is located.
    pub neighbourhood_code: Option<StringValue<LocationNeighbourhoodCode>>,

    /// Any associated website for the polling station location
    pub website: Option<StringValue<WebsiteType>>,

    /// The opening times of the polling station
    pub opening_times: Option<OpeningTimes>,

    /// The RD (Rijksdriehoek) geographic location of the polling station
    pub rd_geographic_location: Option<RDGeographicLocation>,

    /// The geographic location of the polling station
    pub geographic_location: Option<GeographicLocation>,

    /// Whether the polling station is a location where counting takes place
    pub counting_location: Option<bool>,

    /// Accessibility information about the polling station location
    pub accessibility: Option<Accessibility>,

    /// Other information about the polling station location
    pub other_info: Option<Box<str>>,
}

impl Location {
    /// Creates a new `Location` with nothing set yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the BAG ID of the polling station location
    pub fn with_bag_id(self, bag_id: impl Into<LocationBagId>) -> Self {
        Self {
            bag_id: Some(StringValue::from_value(bag_id.into())),
            ..self
        }
    }

    /// Optionally sets the BAG ID of the polling station location
    pub fn with_bag_id_option(self, bag_id: Option<impl Into<LocationBagId>>) -> Self {
        Self {
            bag_id: bag_id.map(|id| StringValue::from_value(id.into())),
            ..self
        }
    }

    /// Sets the street name of the polling station location
    pub fn with_street_name(self, street_name: impl Into<Box<str>>) -> Self {
        Self {
            street_name: Some(street_name.into()),
            ..self
        }
    }

    /// Optionally sets the street name of the polling station location
    pub fn with_street_name_option(self, street_name: Option<impl Into<Box<str>>>) -> Self {
        Self {
            street_name: street_name.map(|name| name.into()),
            ..self
        }
    }

    /// Sets the number of the polling station location
    pub fn with_number(self, number: impl Into<i64>) -> Self {
        Self {
            number: Some(StringValue::from_value(number.into())),
            ..self
        }
    }

    /// Optionally sets the number of the polling station location
    pub fn with_number_option(self, number: Option<impl Into<i64>>) -> Self {
        Self {
            number: number.map(|num| StringValue::from_value(num.into())),
            ..self
        }
    }

    /// Sets the letter of the polling station location
    pub fn with_letter(self, letter: impl Into<Box<str>>) -> Self {
        Self {
            letter: Some(letter.into()),
            ..self
        }
    }

    /// Optionally sets the letter of the polling station location
    pub fn with_letter_option(self, letter: Option<impl Into<Box<str>>>) -> Self {
        Self {
            letter: letter.map(|l| l.into()),
            ..self
        }
    }

    /// Sets the number addition of the polling station location
    pub fn with_number_addition(self, number_addition: impl Into<Box<str>>) -> Self {
        Self {
            number_addition: Some(number_addition.into()),
            ..self
        }
    }

    /// Optionally sets the number addition of the polling station location
    pub fn with_number_addition_option(self, number_addition: Option<impl Into<Box<str>>>) -> Self {
        Self {
            number_addition: number_addition.map(|n| n.into()),
            ..self
        }
    }

    /// Sets the postal code of the polling station location
    pub fn with_postal_code(self, postal_code: impl Into<LocationPostalCode>) -> Self {
        Self {
            postal_code: Some(StringValue::from_value(postal_code.into())),
            ..self
        }
    }

    /// Optionally sets the postal code of the polling station location
    pub fn with_postal_code_option(
        self,
        postal_code: Option<impl Into<LocationPostalCode>>,
    ) -> Self {
        Self {
            postal_code: postal_code.map(|p| StringValue::from_value(p.into())),
            ..self
        }
    }

    /// Sets the city of the polling station location
    pub fn with_city(self, city: impl Into<Box<str>>) -> Self {
        Self {
            city: Some(city.into()),
            ..self
        }
    }

    /// Optionally sets the city of the polling station location
    pub fn with_city_option(self, city: Option<impl Into<Box<str>>>) -> Self {
        Self {
            city: city.map(|c| c.into()),
            ..self
        }
    }

    /// Sets the additional address information of the polling station location
    pub fn with_additional_address_information(
        self,
        additional_address_information: impl Into<Box<str>>,
    ) -> Self {
        Self {
            additional_address_information: Some(additional_address_information.into()),
            ..self
        }
    }

    /// Optionally sets the additional address information of the polling station location
    pub fn with_additional_address_information_option(
        self,
        additional_address_information: Option<impl Into<Box<str>>>,
    ) -> Self {
        Self {
            additional_address_information: additional_address_information.map(|a| a.into()),
            ..self
        }
    }

    /// Sets the building usage of the polling station location
    pub fn with_building_usage(self, building_usage: impl Into<BuildingUsage>) -> Self {
        Self {
            building_usage: Some(StringValue::from_value(building_usage.into())),
            ..self
        }
    }

    /// Optionally sets the building usage of the polling station location
    pub fn with_building_usage_option(
        self,
        building_usage: Option<impl Into<BuildingUsage>>,
    ) -> Self {
        Self {
            building_usage: building_usage.map(|b| StringValue::from_value(b.into())),
            ..self
        }
    }

    /// Sets the district name of the polling station location
    pub fn with_district_name(self, district_name: impl Into<Box<str>>) -> Self {
        Self {
            district_name: Some(district_name.into()),
            ..self
        }
    }

    /// Optionally sets the district name of the polling station location
    pub fn with_district_name_option(self, district_name: Option<impl Into<Box<str>>>) -> Self {
        Self {
            district_name: district_name.map(|d| d.into()),
            ..self
        }
    }

    /// Sets the district code of the polling station location
    pub fn with_district_code(self, district_code: impl Into<LocationDistrictCode>) -> Self {
        Self {
            district_code: Some(StringValue::from_value(district_code.into())),
            ..self
        }
    }

    /// Optionally sets the district code of the polling station location
    pub fn with_district_code_option(
        self,
        district_code: Option<impl Into<LocationDistrictCode>>,
    ) -> Self {
        Self {
            district_code: district_code.map(|d| StringValue::from_value(d.into())),
            ..self
        }
    }

    /// Sets the neighbourhood name of the polling station location
    pub fn with_neighbourhood_name(self, neighbourhood_name: impl Into<Box<str>>) -> Self {
        Self {
            neighbourhood_name: Some(neighbourhood_name.into()),
            ..self
        }
    }

    /// Optionally sets the neighbourhood name of the polling station location
    pub fn with_neighbourhood_name_option(
        self,
        neighbourhood_name: Option<impl Into<Box<str>>>,
    ) -> Self {
        Self {
            neighbourhood_name: neighbourhood_name.map(|n| n.into()),
            ..self
        }
    }

    /// Sets the neighbourhood code of the polling station location
    pub fn with_neighbourhood_code(
        self,
        neighbourhood_code: impl Into<LocationNeighbourhoodCode>,
    ) -> Self {
        Self {
            neighbourhood_code: Some(StringValue::from_value(neighbourhood_code.into())),
            ..self
        }
    }

    /// Optionally sets the neighbourhood code of the polling station location
    pub fn with_neighbourhood_code_option(
        self,
        neighbourhood_code: Option<impl Into<LocationNeighbourhoodCode>>,
    ) -> Self {
        Self {
            neighbourhood_code: neighbourhood_code.map(|c| StringValue::from_value(c.into())),
            ..self
        }
    }

    /// Sets the website of the polling station location
    pub fn with_website(self, website: impl Into<WebsiteType>) -> Self {
        Self {
            website: Some(StringValue::from_value(website.into())),
            ..self
        }
    }

    /// Optionally sets the website of the polling station location
    pub fn with_website_option(self, website: Option<impl Into<WebsiteType>>) -> Self {
        Self {
            website: website.map(|w| StringValue::from_value(w.into())),
            ..self
        }
    }

    /// Sets the opening times of the polling station location
    pub fn with_opening_times(self, opening_times: impl Into<OpeningTimes>) -> Self {
        Self {
            opening_times: Some(opening_times.into()),
            ..self
        }
    }

    /// Optionally sets the opening times of the polling station location
    pub fn with_opening_times_option(self, opening_times: Option<impl Into<OpeningTimes>>) -> Self {
        Self {
            opening_times: opening_times.map(|t| t.into()),
            ..self
        }
    }

    /// Sets the RD geographic location of the polling station location
    pub fn with_rd_geographic_location(
        self,
        rd_geographic_location: impl Into<RDGeographicLocation>,
    ) -> Self {
        Self {
            rd_geographic_location: Some(rd_geographic_location.into()),
            ..self
        }
    }

    /// Optionally sets the RD geographic location of the polling station location
    pub fn with_rd_geographic_location_option(
        self,
        rd_geographic_location: Option<impl Into<RDGeographicLocation>>,
    ) -> Self {
        Self {
            rd_geographic_location: rd_geographic_location.map(|l| l.into()),
            ..self
        }
    }

    /// Sets the geographic location of the polling station location
    pub fn with_geographic_location(
        self,
        geographic_location: impl Into<GeographicLocation>,
    ) -> Self {
        Self {
            geographic_location: Some(geographic_location.into()),
            ..self
        }
    }

    /// Optionally sets the geographic location of the polling station location
    pub fn with_geographic_location_option(
        self,
        geographic_location: Option<impl Into<GeographicLocation>>,
    ) -> Self {
        Self {
            geographic_location: geographic_location.map(|l| l.into()),
            ..self
        }
    }

    /// Sets the counting location of the polling station location
    pub fn with_counting_location(self, counting_location: bool) -> Self {
        Self {
            counting_location: Some(counting_location),
            ..self
        }
    }

    /// Optionally sets the counting location of the polling station location
    pub fn with_counting_location_option(self, counting_location: Option<bool>) -> Self {
        Self {
            counting_location,
            ..self
        }
    }

    /// Sets the accessibility of the polling station location
    pub fn with_accessibility(self, accessibility: impl Into<Accessibility>) -> Self {
        Self {
            accessibility: Some(accessibility.into()),
            ..self
        }
    }

    /// Optionally sets the accessibility of the polling station location
    pub fn with_accessibility_option(
        self,
        accessibility: Option<impl Into<Accessibility>>,
    ) -> Self {
        Self {
            accessibility: accessibility.map(|a| a.into()),
            ..self
        }
    }

    /// Sets the other info of the polling station location
    pub fn with_other_info(self, other_info: impl Into<Box<str>>) -> Self {
        Self {
            other_info: Some(other_info.into()),
            ..self
        }
    }

    /// Optionally sets the other info of the polling station location
    pub fn with_other_info_option(self, other_info: Option<impl Into<Box<str>>>) -> Self {
        Self {
            other_info: other_info.map(|i| i.into()),
            ..self
        }
    }
}

impl EMLElement for Location {
    const EML_NAME: QualifiedName<'_, '_> = QualifiedName::from_static("Location", Some(NS_SB));
    const EML_VERSIONS: EMLVersionRange = EMLVersionRange::since(EMLVersion::V1_3);

    fn read_eml(elem: &mut EMLElementReader<'_, '_>) -> Result<Self, crate::EMLError> {
        struct InternalLocation {
            bag_id: Option<StringValue<LocationBagId>>,
            street_name: Option<Box<str>>,
            number: Option<StringValue<i64>>,
            letter: Option<Box<str>>,
            number_addition: Option<Box<str>>,
            postal_code: Option<StringValue<LocationPostalCode>>,
            city: Option<Box<str>>,
            additional_address_information: Option<Box<str>>,
            building_usage: Option<StringValue<BuildingUsage>>,
            district_name: Option<Box<str>>,
            district_code: Option<StringValue<LocationDistrictCode>>,
            neighbourhood_name: Option<Box<str>>,
            neighbourhood_code: Option<StringValue<LocationNeighbourhoodCode>>,
            website: Option<StringValue<WebsiteType>>,
            opening_times_open: Option<StringValue<XsDateTime>>,
            opening_times_close: Option<StringValue<XsDateTime>>,
            rd_geographic_location_x: Option<StringValue<RDCoordinate>>,
            rd_geographic_location_y: Option<StringValue<RDCoordinate>>,
            geographic_location_latitude: Option<StringValue<Coordinate>>,
            geographic_location_longitude: Option<StringValue<Coordinate>>,
            counting_location: Option<bool>,
            accessibility: Option<Accessibility>,
            other_info: Option<Box<str>>,
        }

        let data = collect_struct!(elem, InternalLocation {
            bag_id as Option: ("BAGId", NS_SB) => |elem| elem.string_value()?,
            street_name as Option: ("StreetName", NS_SB) => |elem| elem.text_without_children()?,
            number as Option: ("Number", NS_SB) => |elem| elem.string_value()?,
            letter as Option: ("Letter", NS_SB) => |elem| elem.text_without_children()?,
            number_addition as Option: ("NumberAddition", NS_SB) => |elem| elem.text_without_children()?,
            postal_code as Option: ("PostalCode", NS_SB) => |elem| elem.string_value()?,
            city as Option: ("City", NS_SB) => |elem| elem.text_without_children()?,
            additional_address_information as Option: ("AdditionalAddressInformation", NS_SB) => |elem| elem.text_without_children()?,
            building_usage as Option: ("BuildingUsage", NS_SB) => |elem| elem.string_value()?,
            district_name as Option: ("DistrictName", NS_SB) => |elem| elem.text_without_children()?,
            district_code as Option: ("DistrictCode", NS_SB) => |elem| elem.string_value()?,
            neighbourhood_name as Option: ("NeighbourhoodName", NS_SB) => |elem| elem.text_without_children()?,
            neighbourhood_code as Option: ("NeighbourhoodCode", NS_SB) => |elem| elem.string_value()?,
            website as Option: ("Website", NS_SB) => |elem| elem.string_value()?,
            opening_times_open as Option: ("OpenTime", NS_SB) => |elem| elem.string_value()?,
            opening_times_close as Option: ("ClosingTime", NS_SB) => |elem| elem.string_value()?,
            rd_geographic_location_x as Option: ("RDx", NS_SB) => |elem| elem.string_value()?,
            rd_geographic_location_y as Option: ("RDy", NS_SB) => |elem| elem.string_value()?,
            geographic_location_latitude as Option: ("Latitude", NS_SB) => |elem| elem.string_value()?,
            geographic_location_longitude as Option: ("Longitude", NS_SB) => |elem| elem.string_value()?,
            counting_location as Option: ("CountingLocation", NS_SB) => |elem| elem.read_bool()?,
            accessibility as Option: Accessibility::EML_NAME => |elem| elem.read_element::<Accessibility>()?,
            other_info as Option: ("OtherInfo", NS_SB) => |elem| elem.text_without_children()?,
        });

        let opening_times = match (data.opening_times_open, data.opening_times_close) {
            (Some(open), Some(close)) => Some(OpeningTimes { open, close }),
            (Some(_), None) => {
                let err = EMLErrorKind::MissingElement(OwnedQualifiedName::from_static(
                    "ClosingTime",
                    Some(NS_SB),
                ))
                .with_span(elem.full_span());
                if elem.parsing_mode() == EMLParsingMode::Strict {
                    return Err(err);
                } else {
                    elem.push_err(err);
                    None
                }
            }
            (None, Some(_)) => {
                let err = EMLErrorKind::MissingElement(OwnedQualifiedName::from_static(
                    "OpenTime",
                    Some(NS_SB),
                ))
                .with_span(elem.full_span());
                if elem.parsing_mode() == EMLParsingMode::Strict {
                    return Err(err);
                } else {
                    elem.push_err(err);
                    None
                }
            }
            _ => None,
        };

        let rd_geographic_location =
            match (data.rd_geographic_location_x, data.rd_geographic_location_y) {
                (Some(x), Some(y)) => Some(RDGeographicLocation { x, y }),
                (Some(_), None) => {
                    let err = EMLErrorKind::MissingElement(OwnedQualifiedName::from_static(
                        "RDy",
                        Some(NS_SB),
                    ))
                    .with_span(elem.full_span());
                    if elem.parsing_mode() == EMLParsingMode::Strict {
                        return Err(err);
                    } else {
                        elem.push_err(err);
                        None
                    }
                }
                (None, Some(_)) => {
                    let err = EMLErrorKind::MissingElement(OwnedQualifiedName::from_static(
                        "RDx",
                        Some(NS_SB),
                    ))
                    .with_span(elem.full_span());
                    if elem.parsing_mode() == EMLParsingMode::Strict {
                        return Err(err);
                    } else {
                        elem.push_err(err);
                        None
                    }
                }
                _ => None,
            };

        let geographic_location = match (
            data.geographic_location_latitude,
            data.geographic_location_longitude,
        ) {
            (Some(latitude), Some(longitude)) => Some(GeographicLocation {
                latitude,
                longitude,
            }),
            (Some(_), None) => {
                let err = EMLErrorKind::MissingElement(OwnedQualifiedName::from_static(
                    "Longitude",
                    Some(NS_SB),
                ))
                .with_span(elem.full_span());
                if elem.parsing_mode() == EMLParsingMode::Strict {
                    return Err(err);
                } else {
                    elem.push_err(err);
                    None
                }
            }
            (None, Some(_)) => {
                let err = EMLErrorKind::MissingElement(OwnedQualifiedName::from_static(
                    "Latitude",
                    Some(NS_SB),
                ))
                .with_span(elem.full_span());
                if elem.parsing_mode() == EMLParsingMode::Strict {
                    return Err(err);
                } else {
                    elem.push_err(err);
                    None
                }
            }
            _ => None,
        };

        Ok(Location {
            bag_id: data.bag_id,
            street_name: data.street_name,
            number: data.number,
            letter: data.letter,
            number_addition: data.number_addition,
            postal_code: data.postal_code,
            city: data.city,
            additional_address_information: data.additional_address_information,
            building_usage: data.building_usage,
            district_name: data.district_name,
            district_code: data.district_code,
            neighbourhood_name: data.neighbourhood_name,
            neighbourhood_code: data.neighbourhood_code,
            website: data.website,
            opening_times,
            rd_geographic_location,
            geographic_location,
            counting_location: data.counting_location,
            accessibility: data.accessibility,
            other_info: data.other_info,
        })
    }

    fn write_eml(&self, writer: crate::io::EMLElementWriter) -> Result<(), crate::EMLError> {
        writer
            .child_option(("BAGId", NS_SB), self.bag_id.as_ref(), |writer, value| {
                writer.text(value.raw().as_ref())?.finish()
            })?
            .child_option(
                ("StreetName", NS_SB),
                self.street_name.as_ref(),
                |writer, value| writer.text(value.as_ref())?.finish(),
            )?
            .child_option(("Number", NS_SB), self.number.as_ref(), |writer, value| {
                writer.text(value.raw().as_ref())?.finish()
            })?
            .child_option(("Letter", NS_SB), self.letter.as_ref(), |writer, value| {
                writer.text(value.as_ref())?.finish()
            })?
            .child_option(
                ("NumberAddition", NS_SB),
                self.number_addition.as_ref(),
                |writer, value| writer.text(value.as_ref())?.finish(),
            )?
            .child_option(
                ("PostalCode", NS_SB),
                self.postal_code.as_ref(),
                |writer, value| writer.text(value.raw().as_ref())?.finish(),
            )?
            .child_option(("City", NS_SB), self.city.as_ref(), |writer, value| {
                writer.text(value.as_ref())?.finish()
            })?
            .child_option(
                ("AdditionalAddressInformation", NS_SB),
                self.additional_address_information.as_ref(),
                |writer, value| writer.text(value.as_ref())?.finish(),
            )?
            .child_option(
                ("BuildingUsage", NS_SB),
                self.building_usage.as_ref(),
                |writer, value| writer.text(value.raw().as_ref())?.finish(),
            )?
            .child_option(
                ("DistrictName", NS_SB),
                self.district_name.as_ref(),
                |writer, value| writer.text(value.as_ref())?.finish(),
            )?
            .child_option(
                ("DistrictCode", NS_SB),
                self.district_code.as_ref(),
                |writer, value| writer.text(value.raw().as_ref())?.finish(),
            )?
            .child_option(
                ("NeighbourhoodName", NS_SB),
                self.neighbourhood_name.as_ref(),
                |writer, value| writer.text(value.as_ref())?.finish(),
            )?
            .child_option(
                ("NeighbourhoodCode", NS_SB),
                self.neighbourhood_code.as_ref(),
                |writer, value| writer.text(value.raw().as_ref())?.finish(),
            )?
            .child_option(
                ("Website", NS_SB),
                self.website.as_ref(),
                |writer, value| writer.text(value.raw().as_ref())?.finish(),
            )?
            .child_option(
                ("OpenTime", NS_SB),
                self.opening_times.as_ref().map(|ot| &ot.open),
                |writer, value| writer.text(value.raw().as_ref())?.finish(),
            )?
            .child_option(
                ("ClosingTime", NS_SB),
                self.opening_times.as_ref().map(|ot| &ot.close),
                |writer, value| writer.text(value.raw().as_ref())?.finish(),
            )?
            .child_option(
                ("RDx", NS_SB),
                self.rd_geographic_location.as_ref().map(|gl| &gl.x),
                |writer, value| writer.text(value.raw().as_ref())?.finish(),
            )?
            .child_option(
                ("RDy", NS_SB),
                self.rd_geographic_location.as_ref().map(|gl| &gl.y),
                |writer, value| writer.text(value.raw().as_ref())?.finish(),
            )?
            .child_option(
                ("Latitude", NS_SB),
                self.geographic_location.as_ref().map(|gl| &gl.latitude),
                |writer, value| writer.text(value.raw().as_ref())?.finish(),
            )?
            .child_option(
                ("Longitude", NS_SB),
                self.geographic_location.as_ref().map(|gl| &gl.longitude),
                |writer, value| writer.text(value.raw().as_ref())?.finish(),
            )?
            .child_option(
                ("CountingLocation", NS_SB),
                self.counting_location,
                |writer, value| writer.bool(value),
            )?
            .child_elem_option(Accessibility::EML_NAME, self.accessibility.as_ref())?
            .child_option(
                ("OtherInfo", NS_SB),
                self.other_info.as_ref(),
                |writer, value| writer.text(value.as_ref())?.finish(),
            )?
            .finish()
    }
}

/// The opening times of a polling station
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpeningTimes {
    /// The opening time of the polling station
    pub open: StringValue<XsDateTime>,

    /// The closing time of the polling station
    pub close: StringValue<XsDateTime>,
}

impl OpeningTimes {
    /// Creates a new [`OpeningTimes`] instance with the given opening and closing times.
    pub fn new(open: impl Into<XsDateTime>, close: impl Into<XsDateTime>) -> Self {
        Self {
            open: StringValue::from_value(open.into()),
            close: StringValue::from_value(close.into()),
        }
    }
}
