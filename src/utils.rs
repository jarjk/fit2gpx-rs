use crate::fit::TrackPoint;
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use std::io::Write;
use std::{fs::File, io::BufWriter, path::Path};
use time::format_description::well_known::Rfc3339;

/// write the `points` to `fname` as gpx
/// # Errors
/// can't write
pub fn write_gpx_to_file(points: &[TrackPoint], fname: impl AsRef<Path>) -> crate::Res<()> {
    // Create file at `fname`
    let gpx_file = File::create(fname.as_ref())?;
    let bufw = BufWriter::new(gpx_file);
    let mut writer = Writer::new_with_indent(bufw, b' ', 2);

    // Write to file
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    let mut gpx = BytesStart::new("gpx");
    gpx.push_attribute(("version", "1.1"));
    gpx.push_attribute(("creator", "fit2gpx"));
    gpx.push_attribute(("xmlns", "http://www.topografix.com/GPX/1/1"));
    gpx.push_attribute((
        "xmlns:gpxtpx",
        "http://www.garmin.com/xmlschemas/TrackPointExtension/v1",
    ));
    gpx.push_attribute(("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"));
    gpx.push_attribute((
        "xsi:schemaLocation",
        "http://www.topografix.com/GPX/1/1 http://www.topografix.com/GPX/1/1/gpx.xsd http://www.garmin.com/xmlschemas/TrackPointExtension/v1 http://www.garmin.com/xmlschemas/TrackPointExtensionv1.xsd",
    ));
    writer.write_event(Event::Start(gpx))?;

    writer.write_event(Event::Start(BytesStart::new("trk")))?;
    writer.write_event(Event::Start(BytesStart::new("trkseg")))?;

    for point in points {
        write_track_point(&mut writer, point)?;
    }

    writer.write_event(Event::End(BytesEnd::new("trkseg")))?;
    writer.write_event(Event::End(BytesEnd::new("trk")))?;
    writer.write_event(Event::End(BytesEnd::new("gpx")))?;
    writer.get_mut().flush()?;

    log::debug!("written {:?}", fname.as_ref());
    Ok(())
}
/// is this `wp` null(0; 0)?
pub fn is_00(wp: &TrackPoint) -> bool {
    let res = (wp.lon, wp.lat) == (0., 0.);
    if res {
        log::trace!("{wp:?} is null");
    }
    res
}

fn write_track_point<W: Write>(writer: &mut Writer<W>, point: &TrackPoint) -> crate::Res<()> {
    let lat = point.lat.to_string();
    let lon = point.lon.to_string();

    let mut trkpt = BytesStart::new("trkpt");
    trkpt.push_attribute(("lat", lat.as_str()));
    trkpt.push_attribute(("lon", lon.as_str()));
    writer.write_event(Event::Start(trkpt))?;

    if let Some(elevation) = point.elevation {
        write_text_element(writer, "ele", &elevation.to_string())?;
    }
    if let Some(time) = point.time {
        write_text_element(writer, "time", &time.format(&Rfc3339)?)?;
    }
    if let Some(speed) = point.speed {
        write_text_element(writer, "speed", &speed.to_string())?;
    }

    if point.heart_rate.is_some()
        || point.cadence.is_some()
        || point.temperature.is_some()
        || point.power.is_some()
        || point.distance.is_some()
    {
        writer.write_event(Event::Start(BytesStart::new("extensions")))?;
        writer.write_event(Event::Start(BytesStart::new("gpxtpx:TrackPointExtension")))?;

        if let Some(hr) = point.heart_rate {
            write_text_element(writer, "gpxtpx:hr", &hr.to_string())?;
        }
        if let Some(cad) = point.cadence {
            write_text_element(writer, "gpxtpx:cad", &cad.to_string())?;
        }
        if let Some(temp) = point.temperature {
            write_text_element(writer, "gpxtpx:atemp", &temp.to_string())?;
        }
        if let Some(power) = point.power {
            write_text_element(writer, "gpxtpx:power", &power.to_string())?;
        }
        if let Some(distance) = point.distance {
            write_text_element(writer, "gpxtpx:distance", &distance.to_string())?;
        }

        writer.write_event(Event::End(BytesEnd::new("gpxtpx:TrackPointExtension")))?;
        writer.write_event(Event::End(BytesEnd::new("extensions")))?;
    }

    writer.write_event(Event::End(BytesEnd::new("trkpt")))?;
    Ok(())
}

fn write_text_element<W: Write>(writer: &mut Writer<W>, name: &str, value: &str) -> crate::Res<()> {
    writer.write_event(Event::Start(BytesStart::new(name)))?;
    writer.write_event(Event::Text(BytesText::new(value)))?;
    writer.write_event(Event::End(BytesEnd::new(name)))?;
    Ok(())
}
