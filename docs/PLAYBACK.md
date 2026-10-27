# Shared playback

The existing nyx-playback crate lives with GTK components because it depends on
nyx-ui. Media Player, Theater, and Pulse pin this same Design revision. Core stays
independent of Design, avoiding repository cycles and duplicate Git crate sources.
No playback API, controls, or application behavior changes are introduced.
