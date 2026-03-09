package SysMLCAPA {
    doc /*
     * Domain library for corrective and preventive action management.
     * Provides definitions for nonconformance tracking, severity
     * classification, disposition, and CAPA workflow states.
     */

    private import ScalarValues::*;

    enum def NonconformanceCategory {
        doc /* Classifies a nonconformance by the type of defect observed. */
        enum dimensional;
        enum material;
        enum cosmetic;
        enum functional;
        enum workmanship;
        enum documentation;
        enum labeling;
        enum packaging;
        enum contamination;
        enum software;
    }

    enum def SeverityClass {
        doc /* Rates the severity of a nonconformance for risk prioritization. */
        enum critical;
        enum major;
        enum minor;
        enum observation;
    }

    enum def Disposition {
        doc /* The material review board decision for a nonconforming item. */
        enum useAsIs;
        enum rework;
        enum repair;
        enum scrap;
        enum returnToVendor;
        enum sortAndScreen;
        enum deviate;
    }

    enum def CorrectiveActionType {
        doc /* Categorizes the type of corrective action taken to address a root cause. */
        enum designChange;
        enum processChange;
        enum supplierChange;
        enum toolingChange;
        enum trainingRetraining;
        enum procedureUpdate;
        enum inspectionEnhancement;
        enum containment;
        enum noActionRequired;
    }

    enum def CapaStatus {
        doc /* Tracks a CAPA record through its full lifecycle from initiation to closure. */
        enum initiated;
        enum investigating;
        enum rootCauseIdentified;
        enum actionPlanned;
        enum actionImplemented;
        enum effectivenessVerified;
        enum closed;
        enum closedIneffective;
        enum reopened;
    }
}
