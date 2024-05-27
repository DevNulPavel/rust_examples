use crate::error::ScannerError;
use std::fmt::{Debug, Display, Formatter};

/// Диапазон портов для сканирования
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum PortRange {
    Ordered(Vec<u16>),
    Specific(Vec<u16>),
}

impl PortRange {
    /// Упорядоченный диапазон портов (`min..max`).
    /// Возвращает ошибку `ScannerError::PortRangeMinGreaterThanMax` в случае, если начало диапазона
    /// больше, чем его конечное значение и `ScannerError::PortEqualsZero`, если одно из значений 0.
    /// ```
    /// use qwatro_port_scanner::range::PortRange;
    ///
    /// let range = PortRange::ordered(100, 105).unwrap();
    ///
    /// assert_eq!(
    ///     range.into_iter().collect::<Vec<_>>(),
    ///     vec![100, 101, 102, 103, 104, 105]
    /// );
    /// ```
    pub fn ordered(min: u16, max: u16) -> Result<Self, ScannerError> {
        if min == 0 || max == 0 {
            return Err(ScannerError::PortEqualsZero);
        }

        if min > max {
            return Err(ScannerError::PortRangeMinGreaterThanMax);
        }

        Ok(Self::Ordered((min..=max).collect()))
    }

    /// Специфический набор портов
    /// ```
    /// use qwatro_port_scanner::range::PortRange;
    ///
    /// let range = PortRange::specific(vec![1000, 2000, 3000]);
    ///
    /// assert_eq!(
    ///     range.into_iter().collect::<Vec<_>>(),
    ///     vec![1000, 2000, 3000]
    /// );
    /// ```
    pub fn specific(ports: Vec<u16>) -> Result<Self, ScannerError> {
        if ports.iter().any(|p| *p == 0) {
            return Err(ScannerError::PortEqualsZero);
        };

        Ok(Self::Specific(ports))
    }
}

impl IntoIterator for PortRange {
    type Item = u16;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            PortRange::Ordered(ports) => ports.into_iter(),
            PortRange::Specific(ports) => ports.into_iter(),
        }
    }
}

impl Display for PortRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // write!(f, "({}, {})", self.x, self.y)
        match self {
            PortRange::Ordered(ports) => write!(
                f,
                "({} - {})",
                ports.first().unwrap_or(&0),
                ports.last().unwrap_or(&0)
            ),
            PortRange::Specific(ports) => write!(
                f,
                "({})",
                ports
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::error::ScannerError;
    use crate::range::PortRange;

    #[test]
    fn ordered() {
        let range = PortRange::ordered(100, 105).unwrap();

        assert_eq!(
            range.into_iter().collect::<Vec<_>>(),
            vec![100, 101, 102, 103, 104, 105]
        );
    }

    #[test]
    fn ordered_invalid_range() {
        assert_eq!(
            PortRange::ordered(105, 100),
            Err(ScannerError::PortRangeMinGreaterThanMax)
        );
    }

    #[test]
    fn ordered_zero_port() {
        assert_eq!(PortRange::ordered(0, 0), Err(ScannerError::PortEqualsZero));
    }

    #[test]
    fn specific() {
        let range = PortRange::specific(vec![1000, 2000, 3000]).unwrap();

        assert_eq!(
            range.into_iter().collect::<Vec<_>>(),
            vec![1000, 2000, 3000]
        );
    }

    #[test]
    fn specific_zero_port() {
        assert_eq!(
            PortRange::specific(vec![0, 1000]),
            Err(ScannerError::PortEqualsZero)
        );
    }
}
