# powertop

[![CI](https://github.com/jteer/powertop/workflows/CI/badge.svg)](https://github.com/jteer/powertop/actions/workflows/ci.yml/badge.svg?branch=main)


Yet another Cross-platform graphical process/system monitor for the terminal


## Road Map
- [ ] TUI
    - [ ] Customizable TUI following [the component architecture](https://ratatui.rs/concepts/application-patterns/component-architecture/)
        - [ ] Color Themes / Styled Components
- [ ] System Monitoring Stats
    - [ ] CPU
        - [x] Initial Component
        - [x] Initial Data Collection
        - [ ] Improved Data Cleanup
        - [ ] Improved Component Visual
    - [ ] Network
    - [ ] Memory
    - [ ] Processes
        - [x] Initial Component
        - [x] Initial Data Collection
        - [ ] Improved Data Cleanup
        - [ ] Improved Component Visual
            - [ ] Additional Process Fields
            - [ ] Sorting Columns
            - [ ] Scrollable
        - [ ] Other Data
            - [ ] [Disk Usage](https://docs.rs/sysinfo/0.30.12/sysinfo/struct.DiskUsage.html)
    - [ ] Disks
    - [ ] GPU / Temp
- [ ] Internal
    - [ ] Additional Logging / Tracing 
