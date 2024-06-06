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
    - [ ] Processes
        - [x] Initial Component
        - [x] Initial Data Collection
        - [ ] Improved Component Visual
            - [ ] Additional Process Fields
            - [ ] Sorting Columns
            - [ ] Scrollable
        - [ ] Other Data
            - [ ] [Disk Usage](https://docs.rs/sysinfo/0.30.12/sysinfo/struct.DiskUsage.html)
    - [ ] Disks
        - [x] Initial Component
        - [x] Initial Data Collection
        - [ ] Improved Component Visual
            - [ ] Display space units
            - [ ] Separate Display as pie chart similar to [gtop disk display](https://github.com/aksakalli/gtop/blob/master/README.md)
    - [ ] GPU / Temp
    - [ ] Network
        - [x] Initial Component
        - [x] Initial Data Collection
        - [ ] Improved Component Visual
            - [ ] Unit Conversion
    - [ ] Memory
        - [x] Initial Component
        - [x] Initial Data Collection
        - [ ] Improved Component Visual
            - [ ] Support More Graph Types
                - [ ] Bar Chart
                - [ ] Pie Chart
- [ ] Internal
    - [ ] Additional Logging / Tracing 
    - [ ] Add Unit Testing
