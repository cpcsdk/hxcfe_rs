use hxcfe::{HeadId, Hxcfe, TrackEncoding, TrackId};

#[test]
fn test_track_bits_and_sector_fields() {
    let mut img = Hxcfe::get().load("tests/EXPERTS.HFE").expect("Unable to read tests/EXPERTS.HFE");

    let bits = img.track_bits(0, 0).expect("no track 0 side 0");
    assert!(bits.len > 0);
    assert_eq!(bits.data.len(), bits.len.div_ceil(8));
    assert!(bits.bitrate > 0);
    assert!(img.track_bits(1000, 0).is_none());

    // writing the same cells back is accepted; a wrong length is refused
    img.set_track_bits(0, 0, &bits.data, None).expect("same cells refused");
    assert!(img.set_track_bits(0, 0, &bits.data[1..], None).is_err());
    assert_eq!(img.track_bits(0, 0).unwrap().data, bits.data);

    let sa = img.sector_access().expect("no sector access");
    let sectors = sa
        .all_track_sectors(HeadId::new(0), TrackId::new(0), TrackEncoding::IsoibmMfm)
        .expect("no sectors on track 0");
    assert!(sectors.nb_sectors() > 0);
    for i in 0..sectors.nb_sectors() {
        let s = sectors.sector_config(i);
        assert_eq!(s.cylinder(), 0);
        assert_eq!(s.size_id(), 2, "512-byte sectors");
        assert!(s.header_crc_ok());
        assert_eq!(s.data_mark(), Some(0xFB));
        assert!(s.data_crc_ok());
        assert!(s.start_data_index() > s.start_sector_index());
        assert_eq!(s.read().len(), 512);
    }
}
