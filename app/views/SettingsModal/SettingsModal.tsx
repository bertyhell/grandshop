import React, { FunctionComponent, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { Button } from '../../components/Button/Button';
import { Select } from '../../components/Select/Select';
import { Modal } from '../../components/Modal/Modal';
import { ModalBody } from '../../components/Modal/Modal.slots';
import FormField from '../../components/FormField/FormField';
import i18n, { languageMap } from '../../shared/translations/i18n';
import { setLanguage } from '../../services/renderer-interop';

import './SettingsModal.scss';

export const LANGUAGE_KEY = 'GRANDSHOP_SETTINGS_LANGUAGE';

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
}

const SettingsModal: FunctionComponent<SettingsModalProps> = ({isOpen, onClose}) => {
  const [t] = useTranslation();

  const [selectedLanguage, setSelectedLanguage] = useState<string>(localStorage.getItem(LANGUAGE_KEY) || 'en');

  useEffect(() => {
    setLanguage(selectedLanguage);
  }, [selectedLanguage]);

  const saveSettings = () => {
    localStorage.setItem(LANGUAGE_KEY, selectedLanguage);
    i18n.changeLanguage(selectedLanguage);
    setLanguage(selectedLanguage);
    onClose();
  };

  return (

      <Modal title={t('views/settings-modal/settings-modal___settings')} isOpen={isOpen} onClose={onClose} size="small" className="m-settings-modal">
        <ModalBody>
          <FormField label={t('views/settings-modal/settings-modal___language')}>
            <Select options={languageMap.map(lang => ({
              label: lang.englishName + ' - ' + lang.nativeName,
              value: lang.code
            }))} value={selectedLanguage} onChange={setSelectedLanguage}/>
          </FormField>
          <Button label={t('views/settings-modal/settings-modal___save')} type="primary" onClick={saveSettings}/>
        </ModalBody>
      </Modal>
  );
};

export default SettingsModal;
