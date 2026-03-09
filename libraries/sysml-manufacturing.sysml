package SysMLManufacturing {
    doc /*
     * Domain library for manufacturing process modeling.
     * Provides definitions for process types, parameters,
     * work instructions, and routing sequences.
     */

    private import ScalarValues::*;

    enum def ProcessType {
        doc /* Classifies a manufacturing process by its technology domain. */
        enum machining;
        enum welding;
        enum molding;
        enum casting;
        enum assembly;
        enum heatTreat;
        enum coating;
        enum testing;
        enum packaging;
        enum printing3d;
    }

    action def ProcessDef {
        doc /*
         * A manufacturing process step performed at a work center.
         * Captures the process classification, location, and
         * time estimates for production planning.
         */
        attribute processType : ProcessType;
        attribute workCenter : String;
        attribute setupTimeMinutes : Real;
        attribute cycleTimeMinutes : Real;
    }

    attribute def ProcessParameterDef {
        doc /*
         * A measurable parameter monitored during a process.
         * Defines the nominal value along with upper and lower
         * control limits and specification limits for SPC.
         */
        attribute parameterName : String;
        attribute nominal : Real;
        attribute ucl : Real;
        attribute lcl : Real;
        attribute usl : Real;
        attribute lsl : Real;
        attribute unit : String;
    }

    action def WorkInstructionDef {
        doc /*
         * A single step within a work instruction sequence.
         * Provides the operator with detailed guidance, safety
         * warnings, and an estimated time to complete the step.
         */
        attribute stepNumber : Integer;
        attribute instruction : String;
        attribute safetyWarning : String;
        attribute estimatedTimeMinutes : Real;
    }

    action def RoutingDef {
        doc /*
         * A manufacturing routing that sequences process steps.
         * Identified by revision and effective date for change
         * control within production engineering.
         */
        attribute revision : String;
        attribute effectiveDate : String;
    }
}
