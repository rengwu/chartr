import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ProjectorScreenChartIcon` instead.
 *
 * @example
 * ```svelte
 * <ProjectorScreenChart color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ProjectorScreenChart: Component<IconComponentProps, {}, "">;
type ProjectorScreenChart = ReturnType<typeof ProjectorScreenChart>;
export default ProjectorScreenChart;
