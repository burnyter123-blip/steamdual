//! Generate a Windows 11 `autounattend.xml` (schneegans-style, emitted locally
//! so the app works offline).
//!
//! Crucially we install **into the partition we already created** rather than
//! wiping the disk: `<InstallTo>` selects the existing NTFS partition by
//! disk/partition index and we omit `<DiskConfiguration>` so Setup leaves the
//! GPT (and the shared ESP / SteamOS partitions) untouched. We also bypass the
//! Win11 TPM / Secure Boot / RAM checks via `LabConfig`, which the Deck needs.

pub struct UnattendOpts {
    /// 0-based disk index as Windows Setup sees it (the only disk passed to the VM = 0).
    pub disk_id: u32,
    /// 1-based partition index of the target NTFS partition as Setup sees it.
    pub partition_id: u32,
    pub username: String,
    pub computer_name: String,
    pub locale: String, // e.g. "en-US"
}

impl Default for UnattendOpts {
    fn default() -> Self {
        UnattendOpts {
            disk_id: 0,
            partition_id: 3,
            username: "Deck".into(),
            computer_name: "STEAMDECK-WIN".into(),
            locale: "en-US".into(),
        }
    }
}

/// Render the full `autounattend.xml`.
pub fn generate(o: &UnattendOpts) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<unattend xmlns="urn:schemas-microsoft-com:unattend">
  <!-- ===== windowsPE: bypass Win11 checks, pick the existing partition ===== -->
  <settings pass="windowsPE">
    <component name="Microsoft-Windows-International-Core-WinPE" processorArchitecture="amd64"
               publicKeyToken="31bf3856ad364e35" language="neutral" versionScope="nonSxS">
      <SetupUILanguage><UILanguage>{locale}</UILanguage></SetupUILanguage>
      <InputLocale>{locale}</InputLocale>
      <SystemLocale>{locale}</SystemLocale>
      <UILanguage>{locale}</UILanguage>
      <UserLocale>{locale}</UserLocale>
    </component>
    <component name="Microsoft-Windows-Setup" processorArchitecture="amd64"
               publicKeyToken="31bf3856ad364e35" language="neutral" versionScope="nonSxS">
      <!-- LabConfig: skip TPM 2.0, Secure Boot and RAM requirement checks. -->
      <RunSynchronous>
        <RunSynchronousCommand wcm:action="add" xmlns:wcm="http://schemas.microsoft.com/WMIConfig/2002/State">
          <Order>1</Order><Path>reg add HKLM\SYSTEM\Setup\LabConfig /v BypassTPMCheck /t REG_DWORD /d 1 /f</Path>
        </RunSynchronousCommand>
        <RunSynchronousCommand wcm:action="add" xmlns:wcm="http://schemas.microsoft.com/WMIConfig/2002/State">
          <Order>2</Order><Path>reg add HKLM\SYSTEM\Setup\LabConfig /v BypassSecureBootCheck /t REG_DWORD /d 1 /f</Path>
        </RunSynchronousCommand>
        <RunSynchronousCommand wcm:action="add" xmlns:wcm="http://schemas.microsoft.com/WMIConfig/2002/State">
          <Order>3</Order><Path>reg add HKLM\SYSTEM\Setup\LabConfig /v BypassRAMCheck /t REG_DWORD /d 1 /f</Path>
        </RunSynchronousCommand>
      </RunSynchronous>
      <UserData>
        <AcceptEula>true</AcceptEula>
        <ProductKey><Key></Key></ProductKey>
      </UserData>
      <ImageInstall>
        <OSImage>
          <!-- Install into the EXISTING partition we created; do not reformat the disk. -->
          <InstallTo>
            <DiskID>{disk_id}</DiskID>
            <PartitionID>{partition_id}</PartitionID>
          </InstallTo>
          <InstallToAvailablePartition>false</InstallToAvailablePartition>
        </OSImage>
      </ImageInstall>
    </component>
  </settings>

  <!-- ===== specialize: computer name ===== -->
  <settings pass="specialize">
    <component name="Microsoft-Windows-Shell-Setup" processorArchitecture="amd64"
               publicKeyToken="31bf3856ad364e35" language="neutral" versionScope="nonSxS">
      <ComputerName>{computer_name}</ComputerName>
    </component>
  </settings>

  <!-- ===== oobeSystem: skip OOBE, create a local account ===== -->
  <settings pass="oobeSystem">
    <component name="Microsoft-Windows-Shell-Setup" processorArchitecture="amd64"
               publicKeyToken="31bf3856ad364e35" language="neutral" versionScope="nonSxS">
      <OOBE>
        <HideEULAPage>true</HideEULAPage>
        <HideOEMRegistrationScreen>true</HideOEMRegistrationScreen>
        <HideOnlineAccountScreens>true</HideOnlineAccountScreens>
        <HideWirelessSetupInOOBE>true</HideWirelessSetupInOOBE>
        <ProtectYourPC>3</ProtectYourPC>
        <SkipMachineOOBE>true</SkipMachineOOBE>
        <SkipUserOOBE>true</SkipUserOOBE>
      </OOBE>
      <UserAccounts>
        <LocalAccounts>
          <LocalAccount wcm:action="add" xmlns:wcm="http://schemas.microsoft.com/WMIConfig/2002/State">
            <Name>{username}</Name>
            <Group>Administrators</Group>
            <DisplayName>{username}</DisplayName>
          </LocalAccount>
        </LocalAccounts>
      </UserAccounts>
      <AutoLogon>
        <Enabled>true</Enabled>
        <Username>{username}</Username>
        <LogonCount>1</LogonCount>
      </AutoLogon>
    </component>
  </settings>
</unattend>
"#,
        locale = o.locale,
        disk_id = o.disk_id,
        partition_id = o.partition_id,
        computer_name = o.computer_name,
        username = o.username,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_target_partition_and_bypasses() {
        let xml = generate(&UnattendOpts { disk_id: 0, partition_id: 3, ..Default::default() });
        assert!(xml.contains("<DiskID>0</DiskID>"));
        assert!(xml.contains("<PartitionID>3</PartitionID>"));
        // No DiskConfiguration => Setup won't wipe the disk.
        assert!(!xml.contains("<DiskConfiguration>"));
        // Win11 check bypasses present.
        assert!(xml.contains("BypassTPMCheck"));
        assert!(xml.contains("BypassSecureBootCheck"));
        // OOBE skipped + local admin created.
        assert!(xml.contains("<SkipMachineOOBE>true</SkipMachineOOBE>"));
        assert!(xml.contains("<Group>Administrators</Group>"));
    }

    #[test]
    fn is_well_formed_enough() {
        let xml = generate(&UnattendOpts::default());
        // Balanced root + the three passes we rely on.
        assert!(xml.contains("<unattend") && xml.contains("</unattend>"));
        assert_eq!(xml.matches("<settings pass=").count(), 3);
    }
}
