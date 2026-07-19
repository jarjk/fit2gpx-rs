use crate::{Res, utils};
use embedded_io_adapters::std::FromStd;
use geo_types::{Point, coord};
use gpx::{Gpx, GpxVersion, Track, TrackSegment, Waypoint};
use rustyfit::profile::{mesgdef, typedef};
use rustyfit::{DecoderEvent, StreamingIterator};
use std::path::{Path, PathBuf};
use std::{fs, io};
use time::OffsetDateTime;

/// Fit Context structure. An instance of this will be passed to the parser and ultimately to the callback function so we can use it for whatever.
#[derive(Default, Clone)]
pub struct Fit {
    pub file_name: PathBuf,
    pub track_segment: TrackSegment,
}

// public fns
impl Fit {
    /// add a filename to `self`, create new instance
    #[must_use]
    pub fn with_filename(self, fname: impl Into<PathBuf>) -> Self {
        Fit {
            file_name: fname.into(),
            ..self
        }
    }
    /// create [`Fit`] from the fit file at `fit_path`
    /// # Errors
    /// can't open file at `fit_path`
    /// can't read fit: invalid
    pub fn from_file(fit_path: impl AsRef<Path>) -> Res<Self> {
        let mut file = fs::File::open(&fit_path)?;

        Ok(Self::from_reader(&mut file)?.with_filename(fit_path.as_ref()))
    }

    /// create [`Fit`] from the fit content of `reader`
    /// also deletes (probably) null or invalid `track_segment.points`
    /// # Errors
    /// can't read fit: invalid
    // TODO: support heart-rate, distance, temperature and such extensions, if `gpx` crate does too
    pub fn from_reader(reader: impl io::Read) -> Res<Self> {
        let mut fit = Fit::default();

        let mut bufread = io::BufReader::new(reader);
        let mut dec = rustyfit::Decoder::new();
        let mut stream = dec.stream(FromStd::new(&mut bufread));

        while let Some(event) = stream.next() {
            if let DecoderEvent::Message(mesg) = event?
                && mesg.num == typedef::MesgNum::RECORD
            {
                let rec = mesgdef::Record::from(mesg);

                let Some(xv) = rec.position_long_degrees() else {
                    continue;
                };
                let Some(yv) = rec.position_lat_degrees() else {
                    continue;
                };
                let mut wp = Waypoint::new(Point(coord! { x: xv, y: yv }));

                if let Some(t) = rec.timestamp.unix_timestamp()
                    && let Ok(dt) = OffsetDateTime::from_unix_timestamp(t)
                {
                    wp.time = Some(dt.into());
                }

                wp.elevation = rec
                    .enhanced_altitude_scaled()
                    .or_else(|| rec.altitude_scaled());

                wp.speed = rec.enhanced_speed_scaled().or_else(|| rec.speed_scaled());

                fit.track_segment.points.push(wp);
            }
        }

        Ok(fit)
    }
    /// convert a fit file at `fit_path`, write to `fname`
    /// # Errors
    /// can't read fit
    /// can't write gpx
    pub fn file_to_gpx(fit_path: impl AsRef<Path>, fname: impl AsRef<Path>) -> Res<()> {
        let fit = Fit::from_file(fit_path)?;
        fit.save_to_gpx(fname)
    }

    /// convert fit content from `read` to `fname`
    /// # Errors
    /// can't read fit
    /// can't write gpx
    pub fn reader_to_gpx(read: impl io::Read, fname: impl AsRef<Path>) -> Res<()> {
        let fit = Fit::from_reader(read)?;
        fit.save_to_gpx(fname)
    }

    /// write `self` to it's gpx, with the same filename, but gpx extension
    /// # Errors
    /// can't write gpx
    pub fn save_to_gpx(self, fname: impl AsRef<Path>) -> Res<()> {
        let gpx: Gpx = self.into();
        utils::write_gpx_to_file(gpx, fname)
    }
}

impl From<Fit> for Gpx {
    fn from(fit: Fit) -> Self {
        // Instantiate Gpx struct
        let track = Track {
            segments: vec![fit.track_segment],
            ..Track::default()
        };
        Self {
            version: GpxVersion::Gpx11,
            tracks: vec![track],
            ..Self::default()
        }
    }
}
