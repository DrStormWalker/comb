use crate::input_enum;

input_enum! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub enum RelAxis {
        X => "x" evdev::RelativeAxisType::REL_X,
        Y => "y" evdev::RelativeAxisType::REL_Y,
        Z => "z" evdev::RelativeAxisType::REL_Z,
        RX => "rx" evdev::RelativeAxisType::REL_RX,
        RY => "ry" evdev::RelativeAxisType::REL_RY,
        RZ => "rz" evdev::RelativeAxisType::REL_RZ,
        HWheel => "hwheel" evdev::RelativeAxisType::REL_HWHEEL,
        Dial => "dial" evdev::RelativeAxisType::REL_DIAL,
        Wheel => "wheel" evdev::RelativeAxisType::REL_WHEEL,
        // Misc => "misc" evdev::RelativeAxisType::REL_MISC,
        HiResWheel => "hi_res_wheel" evdev::RelativeAxisType::REL_WHEEL_HI_RES,
        HiResHWheel => "hi_res_hwheel" evdev::RelativeAxisType::REL_HWHEEL_HI_RES,
    }
    impl TryFrom<evdev::RelativeAxisType>;
}

input_enum! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub enum AbsAxis {
        X => "x" evdev::AbsoluteAxisType::ABS_X,
        Y => "y" evdev::AbsoluteAxisType::ABS_Y,
        Z => "z" evdev::AbsoluteAxisType::ABS_Z,
        RX => "rx" evdev::AbsoluteAxisType::ABS_RX,
        RY => "ry" evdev::AbsoluteAxisType::ABS_RY,
        RZ => "rz" evdev::AbsoluteAxisType::ABS_RZ,
        Throttle => "throttle" evdev::AbsoluteAxisType::ABS_THROTTLE,
        Rudder => "rudder" evdev::AbsoluteAxisType::ABS_RUDDER,
        Wheel => "wheel" evdev::AbsoluteAxisType::ABS_WHEEL,
        Gas => "gas" evdev::AbsoluteAxisType::ABS_GAS,
        Brake => "brake" evdev::AbsoluteAxisType::ABS_BRAKE,
        Hat0X => "hat0x" evdev::AbsoluteAxisType::ABS_HAT0X,
        Hat0Y => "hat0y" evdev::AbsoluteAxisType::ABS_HAT0Y,
        Hat1X => "hat1x" evdev::AbsoluteAxisType::ABS_HAT1X,
        Hat1Y => "hat1y" evdev::AbsoluteAxisType::ABS_HAT1Y,
        Hat2X => "hat2x" evdev::AbsoluteAxisType::ABS_HAT2X,
        Hat2Y => "hat2y" evdev::AbsoluteAxisType::ABS_HAT2Y,
        Hat3X => "hat3x" evdev::AbsoluteAxisType::ABS_HAT3X,
        Hat3Y => "hat3y" evdev::AbsoluteAxisType::ABS_HAT3Y,
        Pressure => "pressure" evdev::AbsoluteAxisType::ABS_PRESSURE,
        Distance => "distance" evdev::AbsoluteAxisType::ABS_DISTANCE,
        TiltX => "tilt_x" evdev::AbsoluteAxisType::ABS_TILT_X,
        TiltY => "tilt_y" evdev::AbsoluteAxisType::ABS_TILT_Y,
        ToolWidth => "tool_width" evdev::AbsoluteAxisType::ABS_TOOL_WIDTH,
        Volume => "volume" evdev::AbsoluteAxisType::ABS_VOLUME,
        // Misc => "misc" evdev::AbsoluteAxisType::ABS_MISC,
        MtSlot => "mt_slot" evdev::AbsoluteAxisType::ABS_MT_SLOT,
        MtTouchMajor => "mt_touch_major" evdev::AbsoluteAxisType::ABS_MT_TOUCH_MAJOR,
        MtTouchMinor => "mt_touch_minor" evdev::AbsoluteAxisType::ABS_MT_TOUCH_MINOR,
        MtWidthMajor => "mt_width_major" evdev::AbsoluteAxisType::ABS_MT_WIDTH_MAJOR,
        MtWidthMinor => "mt_width_minor" evdev::AbsoluteAxisType::ABS_MT_WIDTH_MINOR,
        MtOrientation => "mt_orientation" evdev::AbsoluteAxisType::ABS_MT_ORIENTATION,
        MtPositionX => "mt_position_x" evdev::AbsoluteAxisType::ABS_MT_POSITION_X,
        MtPositionY => "mt_position_y" evdev::AbsoluteAxisType::ABS_MT_POSITION_Y,
        MtToolType => "mt_tool_type" evdev::AbsoluteAxisType::ABS_MT_TOOL_TYPE,
        MtBlobId => "mt_blob_id" evdev::AbsoluteAxisType::ABS_MT_BLOB_ID,
        MtTrackingId => "mt_tracking_id" evdev::AbsoluteAxisType::ABS_MT_TRACKING_ID,
        MtPressure => "mt_pressure" evdev::AbsoluteAxisType::ABS_MT_PRESSURE,
        MtDistnace => "mt_distance" evdev::AbsoluteAxisType::ABS_MT_DISTANCE,
        MtToolX => "mt_tool_x" evdev::AbsoluteAxisType::ABS_MT_TOOL_X,
        MtToolY => "mt_tool_y" evdev::AbsoluteAxisType::ABS_MT_TOOL_Y,
    }
    impl TryFrom<evdev::AbsoluteAxisType>;
}
