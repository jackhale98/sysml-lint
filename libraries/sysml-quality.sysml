package SysMLQuality {
    doc /*
     * Domain library for quality management.
     * Provides definitions for inspection planning,
     * quality characteristic classification, and
     * measurement system analysis.
     */

    private import ScalarValues::*;

    enum def InspectionType {
        doc /* Identifies when in the process flow an inspection occurs. */
        enum incoming;
        enum inProcess;
        enum final;
        enum firstArticle;
        enum periodicRequalification;
    }

    enum def CharacteristicClassification {
        doc /* Ranks the importance of a quality characteristic for risk-based inspection. */
        enum critical;
        enum major;
        enum minor;
        enum informational;
    }

    part def QualityCharacteristicDef {
        doc /*
         * A measurable quality characteristic of a part or process.
         * Captures the characteristic name, its criticality
         * classification, and the method used to measure it.
         */
        attribute characteristicName : String;
        attribute classification : CharacteristicClassification;
        attribute measurementMethod : String;
    }

    part def InspectionPlanDef {
        doc /*
         * Defines the inspection strategy for a part or lot.
         * Specifies the type of inspection, the acceptable
         * quality level, and the sampling inspection level.
         */
        attribute planType : InspectionType;
        attribute aqlLevel : Real;
        attribute inspectionLevel : String;
    }

    part def GaugeRRDef {
        doc /*
         * Configuration for a Gauge Repeatability and Reproducibility
         * study. Defines the study design parameters and the
         * acceptable percent tolerance variance threshold.
         */
        attribute numberOfOperators : Integer;
        attribute numberOfParts : Integer;
        attribute numberOfTrials : Integer;
        attribute acceptablePTV : Real;
    }
}
