# Pi Constellation Mapper

A portable, Raspberry Pi-powered way to explore the night sky and create constellations of your own.

The project sits somewhere between a field instrument, a tiny planetarium, and a creative sketchbook. It should help someone orient themselves under the real sky, inspect stars and familiar constellations, then connect stars into new patterns and save the stories they see there.

## Project goals

- **View the sky:** render a clear, responsive star map for a chosen place and time.
- **Create constellations:** select stars, connect them, name the result, and attach notes or a story.
- **Work in the field:** run comfortably on Raspberry Pi hardware with a compact display, physical controls, and portable power.
- **Stay useful offline:** keep core sky data and saved creations available without an internet connection.
- **Grow in layers:** make the software useful on a desktop first, then integrate Pi-specific hardware without tangling the astronomy, interface, and device-control code.

## Experience

The intended experience is calm and immediate: turn the device on, orient the view, recognize what is overhead, and switch into a creative mode when a pattern catches your eye. The interface should favor darkness, large touch or button targets, minimal menus, and preservation of night vision.

Two modes anchor the design:

1. **Explore** — pan and zoom through the visible sky, identify stars, and toggle established constellation lines and labels.
2. **Create** — choose stars, draw connections, name the new constellation, add its story, and save it locally.

## Proposed system shape

```text
Star catalog + time/location
            |
      Sky projection engine
            |
    Explore / Create interface
            |
  Display, buttons, touch, sensors
            |
        Raspberry Pi device
```

## Current hardware

The first prototype is built around:

- **Raspberry Pi 5 (8 GB):** application computer and hardware-integration host
- **Raspberry Pi Touch Display 2 (5-inch):** primary display and touch interface
- **Adafruit Mini GPS:** location and time input for calculating the visible sky
- **Adafruit BNO085 IMU:** device orientation and movement input for aligning the view

Portable power, enclosure design, cooling, and any additional physical controls are still open decisions. The software should keep each sensor behind a small interface so development can continue on a desktop when the hardware is disconnected.

## First milestones

- [ ] Write the product and hardware requirements
- [ ] Build a desktop star-map prototype using a small real-star catalog
- [ ] Implement pan, zoom, time, and location controls
- [ ] Add constellation creation, naming, and local persistence
- [ ] Establish a night-friendly Pi interface and kiosk startup
- [ ] Integrate the Touch Display 2, GPS, and BNO085 on the Pi
- [ ] Prototype the enclosure, cooling, controls, and portable power setup
- [ ] Test outdoors and refine around real viewing conditions

## Design principles

- Offline-first and privacy-friendly
- Readable without ruining dark adaptation
- Fast enough for modest Raspberry Pi hardware
- Keyboard, touch, and physical-control friendly
- Astronomy data separated from presentation and hardware code
- Custom constellations stored in a simple, exportable format

## Open questions

- How should GPS and IMU readings be calibrated, filtered, and reflected in the interface?
- Should manual location and orientation remain available as fallbacks?
- Should the first UI be a local web app, a Python-native interface, or another lightweight stack?
- How much astronomical accuracy does the first prototype need?
- What should a saved constellation contain beyond stars, lines, name, and story?

## Status

Early concept and architecture phase. The next useful step is a software-only sky-view prototype, followed by the smallest possible Raspberry Pi hardware test.
