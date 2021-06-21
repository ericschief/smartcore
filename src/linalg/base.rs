use std::fmt;
use std::fmt::{Debug, Display};
use std::ops::Neg;
use std::ops::Range;

use crate::num::{FloatNumber, Number};
use num_traits::Signed;

pub trait Array<T: Debug + Display + Copy + Sized, S>: Debug {
    fn get(&self, pos: S) -> &T;

    fn shape(&self) -> S;

    fn is_empty(&self) -> bool;

    fn iterator<'b>(&'b self, axis: u8) -> Box<dyn Iterator<Item = &'b T> + 'b>;
}

pub trait MutArray<T: Debug + Display + Copy + Sized, S>: Array<T, S> {
    fn set(&mut self, pos: S, x: T);

    fn iterator_mut<'b>(&'b mut self, axis: u8) -> Box<dyn Iterator<Item = &'b mut T> + 'b>;

    fn swap(&mut self, a: S, b: S)
    where
        S: Copy,
    {
        let t = *self.get(a);
        self.set(a, *self.get(b));
        self.set(b, t);
    }

    fn div_element_mut(&mut self, pos: S, x: T)
    where
        T: Number,
        S: Copy,
    {
        self.set(pos, *self.get(pos) / x);
    }

    fn mul_element_mut(&mut self, pos: S, x: T)
    where
        T: Number,
        S: Copy,
    {
        self.set(pos, *self.get(pos) * x);
    }

    fn add_element_mut(&mut self, pos: S, x: T)
    where
        T: Number,
        S: Copy,
    {
        self.set(pos, *self.get(pos) + x);
    }

    fn sub_element_mut(&mut self, pos: S, x: T)
    where
        T: Number,
        S: Copy,
    {
        self.set(pos, *self.get(pos) - x);
    }

    fn sub_scalar_mut(&mut self, x: T)
    where
        T: Number,
    {
        self.iterator_mut(0).for_each(|v| *v = *v - x);
    }

    fn add_scalar_mut(&mut self, x: T)
    where
        T: Number,
    {
        self.iterator_mut(0).for_each(|v| *v = *v + x);
    }

    fn mul_scalar_mut(&mut self, x: T)
    where
        T: Number,
    {
        self.iterator_mut(0).for_each(|v| *v = *v * x);
    }

    fn div_scalar_mut(&mut self, x: T)
    where
        T: Number,
    {
        self.iterator_mut(0).for_each(|v| *v = *v / x);
    }

    fn add_mut(&mut self, other: &dyn Array<T, S>)
    where
        T: Number,
        S: Eq,
    {
        assert!(
            self.shape() == other.shape(),
            "A and B should have the same shape"
        );
        self.iterator_mut(0)
            .zip(other.iterator(0))
            .for_each(|(a, &b)| *a = *a + b);
    }

    fn sub_mut(&mut self, other: &dyn Array<T, S>)
    where
        T: Number,
        S: Eq,
    {
        assert!(
            self.shape() == other.shape(),
            "A and B should have the same shape"
        );
        self.iterator_mut(0)
            .zip(other.iterator(0))
            .for_each(|(a, &b)| *a = *a - b);
    }

    fn mul_mut(&mut self, other: &dyn Array<T, S>)
    where
        T: Number,
        S: Eq,
    {
        assert!(
            self.shape() == other.shape(),
            "A and B should have the same shape"
        );
        self.iterator_mut(0)
            .zip(other.iterator(0))
            .for_each(|(a, &b)| *a = *a * b);
    }

    fn div_mut(&mut self, other: &dyn Array<T, S>)
    where
        T: Number,
        S: Eq,
    {
        assert!(
            self.shape() == other.shape(),
            "A and B should have the same shape"
        );
        self.iterator_mut(0)
            .zip(other.iterator(0))
            .for_each(|(a, &b)| *a = *a / b);
    }
}

pub trait ArrayView1<T: Debug + Display + Copy + Sized>: Array<T, usize> {
    fn dot(&self, other: &dyn ArrayView1<T>) -> T
    where
        T: Number,
    {
        assert!(
            self.shape() == other.shape(),
            "Can't take dot product. Arrays have different shapes"
        );
        self.iterator(0)
            .zip(other.iterator(0))
            .map(|(s, o)| *s * *o)
            .sum()
    }

    fn sum(&self) -> T
    where
        T: Number,
    {
        self.iterator(0).map(|v| *v).sum()
    }

    fn max(&self) -> T
    where
        T: Number + PartialOrd,
    {
        let max_f = |max: T, v: &T| -> T {
            match T::gt(v, &max) {
                true => *v,
                _ => max,
            }
        };
        self.iterator(0)
            .fold(T::min_value(), |max, x| max_f(max, x))
    }

    fn min(&self) -> T
    where
        T: Number + PartialOrd,
    {
        let min_f = |min: T, v: &T| -> T {
            match T::lt(v, &min) {
                true => *v,
                _ => min,
            }
        };
        self.iterator(0)
            .fold(T::max_value(), |max, x| min_f(max, x))
    }

    fn argmax(&self) -> usize
    where
        T: Number + PartialOrd,
    {
        let mut max = T::min_value();
        let mut max_pos = 0usize;
        for (i, v) in self.iterator(0).enumerate() {
            if T::gt(&v, &max) {
                max = *v;
                max_pos = i;
            }
        }
        max_pos
    }

    fn unique(&self) -> Vec<T>
    where
        T: Number + Ord,
    {
        let mut result: Vec<T> = self.iterator(0).map(|&v| v).collect();
        result.sort();
        result.dedup();
        result
    }

    fn unique_with_indices(&self) -> (Vec<T>, Vec<usize>)
    where
        T: Number + Ord,
    {
        let mut unique: Vec<T> = self.iterator(0).map(|&v| v).collect();
        unique.sort();
        unique.dedup();

        let mut unique_index = Vec::with_capacity(self.shape());
        for idx in 0..self.shape() {
            unique_index.push(unique.iter().position(|v| self.get(idx) == v).unwrap());
        }

        (unique, unique_index)
    }

    fn norm2(&self) -> f64
    where
        T: Number,
    {
        self.iterator(0)
            .fold(0f64, |norm, xi| {
                let xi = xi.to_f64().unwrap();
                norm + xi * xi
            })
            .sqrt()
    }

    fn norm(&self, p: f64) -> f64
    where
        T: Number,
    {
        if p.is_infinite() && p.is_sign_positive() {
            self.iterator(0)
                .map(|x| x.to_f64().unwrap().abs())
                .fold(std::f64::NEG_INFINITY, |a, b| a.max(b))
        } else if p.is_infinite() && p.is_sign_negative() {
            self.iterator(0)
                .map(|x| x.to_f64().unwrap().abs())
                .fold(std::f64::INFINITY, |a, b| a.min(b))
        } else {
            let mut norm = 0f64;

            for xi in self.iterator(0) {
                norm += xi.to_f64().unwrap().abs().powf(p);
            }

            norm.powf(1f64 / p)
        }
    }

    fn max_diff(&self, other: &dyn ArrayView1<T>) -> T
    where
        T: Number + Signed + PartialOrd,
    {
        assert!(
            self.shape() == other.shape(),
            "Both arrays should have the same shape ({})",
            self.shape()
        );
        let max_f = |max: T, v: T| -> T {
            match T::gt(&v, &max) {
                true => v,
                _ => max,
            }
        };
        self.iterator(0)
            .zip(other.iterator(0))
            .map(|(&a, &b)| (a - b).abs())
            .fold(T::min_value(), |max, x| max_f(max, x))
    }

    fn var(&self) -> f64
    where
        T: Number,
    {
        let n = self.shape();

        let mut mu = 0f64;
        let mut sum = 0f64;
        let div = n as f64;
        for i in 0..n {
            let xi = T::to_f64(self.get(i)).unwrap();
            mu += xi;
            sum += xi * xi;
        }
        mu /= div;
        sum / div - mu.powi(2)
    }

    fn std(&self) -> f64
    where
        T: Number,
    {
        self.var().sqrt()
    }

    fn mean(&self) -> f64
    where
        T: Number,
    {
        self.sum().to_f64().unwrap() / self.shape() as f64
    }
}

pub trait ArrayView2<T: Debug + Display + Copy + Sized>: Array<T, (usize, usize)> {
    fn max(&self, axis: u8) -> Vec<T>
    where
        T: Number + PartialOrd,
    {
        let (nrows, ncols) = self.shape();
        let max_f = |max: T, r: usize, c: usize| -> T {
            let v = self.get((r, c));
            match T::gt(v, &max) {
                true => *v,
                _ => max,
            }
        };
        match axis {
            0 => (0..ncols)
                .map(move |c| (0..nrows).fold(T::min_value(), |max, r| max_f(max, r, c)))
                .collect(),
            _ => (0..nrows)
                .map(move |r| (0..ncols).fold(T::min_value(), |max, c| max_f(max, r, c)))
                .collect(),
        }
    }

    fn sum(&self, axis: u8) -> Vec<T>
    where
        T: Number,
    {
        let (nrows, ncols) = self.shape();
        match axis {
            0 => (0..ncols)
                .map(move |c| (0..nrows).map(|r| *self.get((r, c))).sum())
                .collect(),
            _ => (0..nrows)
                .map(move |r| (0..ncols).map(|c| *self.get((r, c))).sum())
                .collect(),
        }
    }

    fn min(&self, axis: u8) -> Vec<T>
    where
        T: Number + PartialOrd,
    {
        let (nrows, ncols) = self.shape();
        let min_f = |min: T, r: usize, c: usize| -> T {
            let v = self.get((r, c));
            match T::lt(v, &min) {
                true => *v,
                _ => min,
            }
        };
        match axis {
            0 => (0..ncols)
                .map(move |c| (0..nrows).fold(T::max_value(), |min, r| min_f(min, r, c)))
                .collect(),
            _ => (0..nrows)
                .map(move |r| (0..ncols).fold(T::max_value(), |min, c| min_f(min, r, c)))
                .collect(),
        }
    }

    fn argmax(&self, axis: u8) -> Vec<usize>
    where
        T: Number + PartialOrd,
    {
        let max_f = |max: (T, usize), v: (T, usize)| -> (T, usize) {
            match T::gt(&v.0, &max.0) {
                true => v,
                _ => max,
            }
        };
        let (nrows, ncols) = self.shape();
        match axis {
            0 => (0..ncols)
                .map(move |c| {
                    (0..nrows).fold((T::min_value(), 0), |max, r| {
                        max_f(max, (*self.get((r, c)), r))
                    })
                })
                .map(|(_, i)| i)
                .collect(),
            _ => (0..nrows)
                .map(move |r| {
                    (0..ncols).fold((T::min_value(), 0), |max, c| {
                        max_f(max, (*self.get((r, c)), c))
                    })
                })
                .map(|(_, i)| i)
                .collect(),
        }
    }

    fn mean(&self, axis: u8) -> Vec<f64>
    where
        T: Number,
    {
        let (n, m) = match axis {
            0 => {
                let (n, m) = self.shape();
                (m, n)
            }
            _ => self.shape(),
        };

        let mut x: Vec<f64> = vec![0f64; n];

        let div = m as f64;

        for (i, x_i) in x.iter_mut().enumerate().take(n) {
            for j in 0..m {
                *x_i += match axis {
                    0 => T::to_f64(self.get((j, i))).unwrap(),
                    _ => T::to_f64(self.get((i, j))).unwrap(),
                };
            }
            *x_i /= div;
        }

        x
    }

    fn var(&self, axis: u8) -> Vec<f64>
    where
        T: Number,
    {
        let (n, m) = match axis {
            0 => {
                let (n, m) = self.shape();
                (m, n)
            }
            _ => self.shape(),
        };

        let mut x: Vec<f64> = vec![0f64; n];

        let div = m as f64;

        for (i, x_i) in x.iter_mut().enumerate().take(n) {
            let mut mu = 0f64;
            let mut sum = 0f64;
            for j in 0..m {
                let a = match axis {
                    0 => T::to_f64(self.get((j, i))).unwrap(),
                    _ => T::to_f64(self.get((i, j))).unwrap(),
                };
                mu += a;
                sum += a * a;
            }
            mu /= div;
            *x_i = sum / div - mu.powi(2);
        }

        x
    }

    fn std(&self, axis: u8) -> Vec<f64>
    where
        T: Number,
    {
        let mut x = self.var(axis);

        let n = match axis {
            0 => self.shape().1,
            _ => self.shape().0,
        };

        for x_i in x.iter_mut().take(n) {
            *x_i = x_i.sqrt();
        }

        x
    }

    fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (nrows, ncols) = self.shape();
        for r in 0..nrows {
            let row: Vec<T> = (0..ncols).map(|c| *self.get((r, c))).collect();
            writeln!(f, "{:?}", row)?
        }
        Ok(())
    }
}

pub trait MutArrayView1<T: Debug + Display + Copy + Sized>:
    MutArray<T, usize> + ArrayView1<T>
{
    fn copy_from(&mut self, other: &dyn Array<T, usize>) {
        self.iterator_mut(0)
            .zip(other.iterator(0))
            .for_each(|(s, o)| *s = *o);
    }

    fn abs_mut(&mut self)
    where
        T: Number + Signed,
    {
        self.iterator_mut(0).for_each(|v| *v = v.abs());
    }

    fn neg_mut(&mut self)
    where
        T: Number + Neg<Output = T>,
    {
        self.iterator_mut(0).for_each(|v| *v = -*v);
    }

    fn pow_mut(&mut self, p: T)
    where
        T: FloatNumber,
    {
        self.iterator_mut(0).for_each(|v| *v = v.powf(p));
    }

    fn argsort_mut(&mut self) -> Vec<usize>
    where
        T: Number + PartialOrd,
    {
        let stack_size = 64;
        let mut jstack = -1;
        let mut l = 0;
        let mut istack = vec![0; stack_size];
        let mut ir = self.shape() - 1;
        let mut index: Vec<usize> = (0..self.shape()).collect();

        loop {
            if ir - l < 7 {
                for j in l + 1..=ir {
                    let a = *self.get(j);
                    let b = index[j];
                    let mut i: i32 = (j - 1) as i32;
                    while i >= l as i32 {
                        if *self.get(i as usize) <= a {
                            break;
                        }
                        self.set((i + 1) as usize, *self.get(i as usize));
                        index[(i + 1) as usize] = index[i as usize];
                        i -= 1;
                    }
                    self.set((i + 1) as usize, a);
                    index[(i + 1) as usize] = b;
                }
                if jstack < 0 {
                    break;
                }
                ir = istack[jstack as usize];
                jstack -= 1;
                l = istack[jstack as usize];
                jstack -= 1;
            } else {
                let k = (l + ir) >> 1;
                self.swap(k, l + 1);
                index.swap(k, l + 1);
                if self.get(l) > self.get(ir) {
                    self.swap(l, ir);
                    index.swap(l, ir);
                }
                if self.get(l + 1) > self.get(ir) {
                    self.swap(l + 1, ir);
                    index.swap(l + 1, ir);
                }
                if self.get(l) > self.get(l + 1) {
                    self.swap(l, l + 1);
                    index.swap(l, l + 1);
                }
                let mut i = l + 1;
                let mut j = ir;
                let a = *self.get(l + 1);
                let b = index[l + 1];
                loop {
                    loop {
                        i += 1;
                        if *self.get(i) >= a {
                            break;
                        }
                    }
                    loop {
                        j -= 1;
                        if *self.get(j) <= a {
                            break;
                        }
                    }
                    if j < i {
                        break;
                    }
                    self.swap(i, j);
                    index.swap(i, j);
                }
                self.set(l + 1, *self.get(j));
                self.set(j, a);
                index[l + 1] = index[j];
                index[j] = b;
                jstack += 2;

                if jstack >= 64 {
                    panic!("stack size is too small.");
                }

                if ir - i + 1 >= j - l {
                    istack[jstack as usize] = ir;
                    istack[jstack as usize - 1] = i;
                    ir = j - 1;
                } else {
                    istack[jstack as usize] = j - 1;
                    istack[jstack as usize - 1] = l;
                    l = i;
                }
            }
        }

        index
    }

    fn softmax_mut(&mut self)
    where
        T: FloatNumber,
    {
        let max = self.max();
        let mut z = T::zero();
        self.iterator_mut(0).for_each(|v| {
            *v = (*v - max).exp();
            z += *v;
        });
        self.iterator_mut(0).for_each(|v| *v = *v / z);
    }
}

pub trait MutArrayView2<T: Debug + Display + Copy + Sized>:
    MutArray<T, (usize, usize)> + ArrayView2<T>
{
    fn copy_from(&mut self, other: &dyn Array<T, (usize, usize)>) {
        self.iterator_mut(0)
            .zip(other.iterator(0))
            .for_each(|(s, o)| *s = *o);
    }

    fn abs_mut(&mut self)
    where
        T: Number + Signed,
    {
        self.iterator_mut(0).for_each(|v| *v = v.abs());
    }

    fn neg_mut(&mut self)
    where
        T: Number + Neg<Output = T>,
    {
        self.iterator_mut(0).for_each(|v| *v = -*v);
    }

    fn pow_mut(&mut self, p: T)
    where
        T: FloatNumber,
    {
        self.iterator_mut(0).for_each(|v| *v = v.powf(p));
    }

    fn scale_mut(&mut self, mean: &[T], std: &[T], axis: u8)
    where
        T: Number,
    {
        let (n, m) = match axis {
            0 => {
                let (n, m) = self.shape();
                (m, n)
            }
            _ => self.shape(),
        };

        for i in 0..n {
            for j in 0..m {
                match axis {
                    0 => self.set((j, i), (*self.get((j, i)) - mean[i]) / std[i]),
                    _ => self.set((i, j), (*self.get((i, j)) - mean[i]) / std[i]),
                }
            }
        }
    }
}

pub trait Array1<T: Debug + Display + Copy + Sized>: MutArrayView1<T> + Sized + Clone {
    fn slice<'a>(&'a self, range: Range<usize>) -> Box<dyn ArrayView1<T> + 'a>;

    fn slice_mut<'a>(&'a mut self, range: Range<usize>) -> Box<dyn MutArrayView1<T> + 'a>;

    fn fill(len: usize, value: T) -> Self;

    fn from_iterator<I: Iterator<Item = T>>(iter: I, len: usize) -> Self;

    fn from_vec_slice(slice: &[T]) -> Self;

    fn from_slice<'a>(slice: &'a dyn ArrayView1<T>) -> Self;

    fn zeros(len: usize) -> Self
    where
        T: Number,
    {
        Self::fill(len, T::zero())
    }
    fn ones(len: usize) -> Self
    where
        T: Number,
    {
        Self::fill(len, T::one())
    }

    fn rand(len: usize) -> Self
    where
        T: FloatNumber,
    {
        Self::from_iterator((0..len).map(|_| T::rand()), len)
    }

    fn add_scalar(&self, x: T) -> Self
    where
        T: Number,
    {
        let mut result = self.clone();
        result.add_scalar_mut(x);
        result
    }

    fn sub_scalar(&self, x: T) -> Self
    where
        T: Number,
    {
        let mut result = self.clone();
        result.sub_scalar_mut(x);
        result
    }

    fn div_scalar(&self, x: T) -> Self
    where
        T: Number,
    {
        let mut result = self.clone();
        result.div_scalar_mut(x);
        result
    }

    fn mul_scalar(&self, x: T) -> Self
    where
        T: Number,
    {
        let mut result = self.clone();
        result.mul_scalar_mut(x);
        result
    }

    fn add(&self, other: &dyn Array<T, usize>) -> Self
    where
        T: Number,
    {
        let mut result = self.clone();
        result.add_mut(other);
        result
    }

    fn sub(&self, other: &dyn Array<T, usize>) -> Self
    where
        T: Number,
    {
        let mut result = self.clone();
        result.sub_mut(other);
        result
    }

    fn mul(&self, other: &dyn Array<T, usize>) -> Self
    where
        T: Number,
    {
        let mut result = self.clone();
        result.mul_mut(other);
        result
    }

    fn div(&self, other: &dyn Array<T, usize>) -> Self
    where
        T: Number,
    {
        let mut result = self.clone();
        result.div_mut(other);
        result
    }

    fn take(&self, index: &[usize]) -> Self {
        let len = self.shape();
        assert!(
            index.iter().all(|&i| i < len),
            "All indices in `take` should be < {}",
            len
        );
        Self::from_iterator(index.iter().map(move |&i| *self.get(i)), index.len())
    }

    fn abs(&self) -> Self
    where
        T: Number + Signed,
    {
        let mut result = self.clone();
        result.abs_mut();
        result
    }

    fn neg(&self) -> Self
    where
        T: Number + Neg<Output = T>,
    {
        let mut result = self.clone();
        result.neg_mut();
        result
    }

    fn pow(&self, p: T) -> Self
    where
        T: FloatNumber,
    {
        let mut result = self.clone();
        result.pow_mut(p);
        result
    }

    fn argsort(&self) -> Vec<usize>
    where
        T: Number + PartialOrd,
    {
        let mut v = self.clone();
        v.argsort_mut()
    }

    fn map<O: Debug + Display + Copy + Sized, A: Array1<O>, F: FnMut(&T) -> O>(self, f: F) -> A {
        let len = self.shape();
        A::from_iterator(self.iterator(0).map(f), len)
    }

    fn softmax(&self) -> Self
    where
        T: FloatNumber,
    {
        let mut result = self.clone();
        result.softmax_mut();
        result
    }
}

pub trait Array2<T: Debug + Display + Copy + Sized>: MutArrayView2<T> + Sized + Clone {
    fn fill(nrows: usize, ncols: usize, value: T) -> Self;

    fn slice<'a>(&'a self, rows: Range<usize>, cols: Range<usize>) -> Box<dyn ArrayView2<T> + 'a>
    where
        Self: Sized;

    fn slice_mut<'a>(
        &'a mut self,
        rows: Range<usize>,
        cols: Range<usize>,
    ) -> Box<dyn MutArrayView2<T> + 'a>
    where
        Self: Sized;

    fn from_iterator<I: Iterator<Item = T>>(iter: I, nrows: usize, ncols: usize, axis: u8) -> Self;

    fn get_row<'a>(&'a self, row: usize) -> Box<dyn ArrayView1<T> + 'a>
    where
        Self: Sized;

    fn get_col<'a>(&'a self, col: usize) -> Box<dyn ArrayView1<T> + 'a>
    where
        Self: Sized;

    fn zeros(nrows: usize, ncols: usize) -> Self
    where
        T: Number,
    {
        Self::fill(nrows, ncols, T::zero())
    }

    fn ones(nrows: usize, ncols: usize) -> Self
    where
        T: Number,
    {
        Self::fill(nrows, ncols, T::one())
    }

    fn eye(size: usize) -> Self
    where
        T: Number,
    {
        let mut matrix = Self::zeros(size, size);

        for i in 0..size {
            matrix.set((i, i), T::one());
        }

        matrix
    }

    fn rand(nrows: usize, ncols: usize) -> Self
    where
        T: FloatNumber,
    {
        Self::from_iterator((0..nrows * ncols).map(|_| T::rand()), nrows, ncols, 0)
    }

    fn from_slice(slice: &dyn ArrayView2<T>) -> Self {
        let (nrows, ncols) = slice.shape();
        Self::from_iterator(slice.iterator(0).cloned(), nrows, ncols, 0)
    }

    fn from_row(slice: &dyn ArrayView1<T>) -> Self {
        let ncols = slice.shape();
        Self::from_iterator(slice.iterator(0).cloned(), 1, ncols, 0)
    }

    fn from_column(slice: &dyn ArrayView1<T>) -> Self {
        let nrows = slice.shape();
        Self::from_iterator(slice.iterator(0).cloned(), nrows, 1, 0)
    }

    fn transpose(&self) -> Self {
        let (nrows, ncols) = self.shape();
        let mut m = Self::fill(ncols, nrows, *self.get((0, 0)));
        for c in 0..ncols {
            for r in 0..nrows {
                m.set((c, r), *self.get((r, c)));
            }
        }
        m
    }

    fn reshape(&self, nrows: usize, ncols: usize, axis: u8) -> Self {
        let (onrows, oncols) = self.shape();

        assert!(
            nrows * ncols == onrows * oncols,
            "Can't reshape {}x{} array into a {}x{} array",
            onrows,
            oncols,
            nrows,
            ncols
        );

        Self::from_iterator(self.iterator(0).cloned(), nrows, ncols, axis)
    }

    fn matmul(&self, other: &dyn ArrayView2<T>) -> Self
    where
        T: Number,
    {
        let (nrows, ncols) = self.shape();
        let (o_nrows, o_ncols) = other.shape();
        if ncols != o_nrows {
            panic!("Number of rows of A should equal number of columns of B");
        }
        let inner_d = ncols;
        let mut result = Self::zeros(nrows, o_ncols);

        for r in 0..nrows {
            for c in 0..o_ncols {
                let mut s = T::zero();
                for i in 0..inner_d {
                    s += *self.get((r, i)) * *other.get((i, c));
                }
                result.set((r, c), s);
            }
        }

        result
    }

    fn ab(&self, a_transpose: bool, b: &Self, b_transpose: bool) -> Self
    where
        T: Number,
    {
        match (a_transpose, b_transpose) {
            (true, true) => b.matmul(self).transpose(),
            (false, true) => self.matmul(&b.transpose()),
            (true, false) => self.transpose().matmul(b),
            (false, false) => self.matmul(b),
        }
    }

    fn concatenate_1d<'a>(arrays: &'a [&'a dyn ArrayView1<T>], axis: u8) -> Self {
        assert!(
            axis == 1 || axis == 0,
            "For two dimensional array `axis` should be either 0 or 1"
        );
        assert!(arrays.len() > 0, "Can't concatenate an empty array");
        assert!(
            arrays.windows(2).all(|w| w[0].shape() == w[1].shape()),
            "Can't concatenate arrays of different sizes"
        );

        let first = &arrays[0];
        let tail = &arrays[1..];

        match axis {
            0 => Self::from_iterator(
                tail.iter()
                    .fold(first.iterator(0), |acc, i| {
                        Box::new(acc.chain(i.iterator(0)))
                    })
                    .cloned(),
                arrays.len(),
                arrays[0].shape(),
                axis,
            ),
            _ => Self::from_iterator(
                tail.iter()
                    .fold(first.iterator(0), |acc, i| {
                        Box::new(acc.chain(i.iterator(0)))
                    })
                    .cloned(),
                arrays[0].shape(),
                arrays.len(),
                axis,
            ),
        }
    }

    fn concatenate_2d<'a>(arrays: &'a [&'a dyn ArrayView2<T>], axis: u8) -> Self {
        assert!(
            axis == 1 || axis == 0,
            "For two dimensional array `axis` should be either 0 or 1"
        );
        assert!(arrays.len() > 0, "Can't concatenate an empty array");
        if axis == 0 {
            assert!(
                arrays.windows(2).all(|w| w[0].shape().1 == w[1].shape().1),
                "Number of columns in all arrays should match"
            );
        } else {
            assert!(
                arrays.windows(2).all(|w| w[0].shape().0 == w[1].shape().0),
                "Number of rows in all arrays should match"
            );
        }

        let first = &arrays[0];
        let tail = &arrays[1..];

        match axis {
            0 => {
                let (nrows, ncols) = (
                    arrays.iter().map(|a| a.shape().0).sum(),
                    arrays[0].shape().1,
                );
                Self::from_iterator(
                    tail.iter()
                        .fold(first.iterator(0), |acc, i| {
                            Box::new(acc.chain(i.iterator(0)))
                        })
                        .cloned(),
                    nrows,
                    ncols,
                    axis,
                )
            }
            _ => {
                let (nrows, ncols) = (
                    arrays[0].shape().0,
                    (arrays.iter().map(|a| a.shape().1).sum()),
                );
                Self::from_iterator(
                    tail.iter()
                        .fold(first.iterator(1), |acc, i| {
                            Box::new(acc.chain(i.iterator(1)))
                        })
                        .cloned(),
                    nrows,
                    ncols,
                    axis,
                )
            }
        }
    }

    fn merge_1d<'a>(&'a self, arrays: &'a [&'a dyn ArrayView1<T>], axis: u8, append: bool) -> Self {
        assert!(
            axis == 1 || axis == 0,
            "For two dimensional array `axis` should be either 0 or 1"
        );
        assert!(arrays.len() > 0, "Can't merge with an empty array");

        let first = &arrays[0];
        let tail = &arrays[1..];

        match (append, axis) {
            (true, 0) => {
                let (nrows, ncols) = (self.shape().0 + arrays.len(), self.shape().1);
                Self::from_iterator(
                    self.iterator(0)
                        .chain(tail.iter().fold(first.iterator(0), |acc, i| {
                            Box::new(acc.chain(i.iterator(0)))
                        }))
                        .cloned(),
                    nrows,
                    ncols,
                    axis,
                )
            }
            (true, 1) => {
                let (nrows, ncols) = (self.shape().0, self.shape().1 + arrays.len());
                Self::from_iterator(
                    self.iterator(1)
                        .chain(tail.iter().fold(first.iterator(0), |acc, i| {
                            Box::new(acc.chain(i.iterator(0)))
                        }))
                        .cloned(),
                    nrows,
                    ncols,
                    axis,
                )
            }
            (false, 0) => {
                let (nrows, ncols) = (self.shape().0 + arrays.len(), self.shape().1);
                Self::from_iterator(
                    tail.iter()
                        .fold(first.iterator(0), |acc, i| {
                            Box::new(acc.chain(i.iterator(0)))
                        })
                        .chain(self.iterator(0))
                        .cloned(),
                    nrows,
                    ncols,
                    axis,
                )
            }
            _ => {
                let (nrows, ncols) = (self.shape().0, self.shape().1 + arrays.len());
                Self::from_iterator(
                    tail.iter()
                        .fold(first.iterator(0), |acc, i| {
                            Box::new(acc.chain(i.iterator(0)))
                        })
                        .chain(self.iterator(1))
                        .cloned(),
                    nrows,
                    ncols,
                    axis,
                )
            }
        }
    }

    fn v_stack(&self, other: &dyn ArrayView2<T>) -> Self {
        let (nrows, ncols) = self.shape();
        let (other_nrows, other_ncols) = other.shape();

        assert!(
            ncols == other_ncols,
            "For vertical stack number of rows in both arrays should match"
        );
        Self::from_iterator(
            self.iterator(0).chain(other.iterator(0)).cloned(),
            nrows + other_nrows,
            ncols,
            0,
        )
    }

    fn h_stack(&self, other: &dyn ArrayView2<T>) -> Self {
        let (nrows, ncols) = self.shape();
        let (other_nrows, other_ncols) = other.shape();

        assert!(
            nrows == other_nrows,
            "For horizontal stack number of rows in both arrays should match"
        );
        Self::from_iterator(
            self.iterator(1).chain(other.iterator(1)).cloned(),
            nrows,
            other_ncols + ncols,
            1,
        )
    }

    fn map<O: Debug + Display + Copy + Sized, A: Array2<O>, F: FnMut(&T) -> O>(self, f: F) -> A {
        let (nrows, ncols) = self.shape();
        A::from_iterator(self.iterator(0).map(f), nrows, ncols, 0)
    }

    fn row_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Box<dyn ArrayView1<T> + 'a>> + 'a> {
        Box::new((0..self.shape().0).map(move |r| self.get_row(r)))
    }

    fn col_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Box<dyn ArrayView1<T> + 'a>> + 'a> {
        Box::new((0..self.shape().1).map(move |r| self.get_col(r)))
    }

    fn take(&self, index: &[usize], axis: u8) -> Self {
        let (nrows, ncols) = self.shape();

        match axis {
            0 => {
                assert!(
                    index.iter().all(|&i| i < nrows),
                    "All indices in `take` should be < {}",
                    nrows
                );
                Self::from_iterator(
                    index
                        .iter()
                        .flat_map(move |&r| (0..ncols).map(move |c| self.get((r, c))))
                        .cloned(),
                    index.len(),
                    ncols,
                    0,
                )
            }
            _ => {
                assert!(
                    index.iter().all(|&i| i < ncols),
                    "All indices in `take` should be < {}",
                    ncols
                );
                Self::from_iterator(
                    (0..nrows)
                        .flat_map(move |r| index.iter().map(move |&c| self.get((r, c))))
                        .cloned(),
                    nrows,
                    index.len(),
                    0,
                )
            }
        }
    }

    fn add_scalar(&self, x: T) -> Self
    where
        T: Number,
    {
        let mut result = self.clone();
        result.add_scalar_mut(x);
        result
    }

    fn sub_scalar(&self, x: T) -> Self
    where
        T: Number,
    {
        let mut result = self.clone();
        result.sub_scalar_mut(x);
        result
    }

    fn div_scalar(&self, x: T) -> Self
    where
        T: Number,
    {
        let mut result = self.clone();
        result.div_scalar_mut(x);
        result
    }

    fn mul_scalar(&self, x: T) -> Self
    where
        T: Number,
    {
        let mut result = self.clone();
        result.mul_scalar_mut(x);
        result
    }

    fn add(&self, other: &dyn Array<T, (usize, usize)>) -> Self
    where
        T: Number,
    {
        let mut result = self.clone();
        result.add_mut(other);
        result
    }

    fn sub(&self, other: &dyn Array<T, (usize, usize)>) -> Self
    where
        T: Number,
    {
        let mut result = self.clone();
        result.sub_mut(other);
        result
    }

    fn mul(&self, other: &dyn Array<T, (usize, usize)>) -> Self
    where
        T: Number,
    {
        let mut result = self.clone();
        result.mul_mut(other);
        result
    }

    fn div(&self, other: &dyn Array<T, (usize, usize)>) -> Self
    where
        T: Number,
    {
        let mut result = self.clone();
        result.div_mut(other);
        result
    }

    fn abs(&self) -> Self
    where
        T: Number + Signed,
    {
        let mut result = self.clone();
        result.abs_mut();
        result
    }

    fn neg(&self) -> Self
    where
        T: Number + Neg<Output = T>,
    {
        let mut result = self.clone();
        result.neg_mut();
        result
    }

    fn pow(&self, p: T) -> Self
    where
        T: FloatNumber,
    {
        let mut result = self.clone();
        result.pow_mut(p);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linalg::dense::matrix::DenseMatrix;
    use approx::relative_eq;

    #[test]
    fn test_dot() {
        let a = vec![1, 2, 3];
        let b = vec![1.0, 2.0, 3.0];
        let c = vec![4.0, 5.0, 6.0];

        assert_eq!(b.slice(0..2).dot(c.slice(0..2).as_ref()), 14.);
        assert_eq!(b.slice(0..3).dot(&c), 32.);
        assert_eq!(b.dot(&c), 32.);
        assert_eq!(a.dot(&a), 14);
    }

    #[test]
    #[should_panic]
    fn test_failed_dot() {
        let a = vec![1, 2, 3];

        a.slice(0..2).dot(a.slice(0..3).as_ref());
    }

    #[test]
    fn test_vec_chaining() {
        let mut x: Vec<i32> = Vec::zeros(6);

        x.add_scalar(5);
        assert_eq!(vec!(5, 5, 5, 5, 5, 5), x.add_scalar(5));
        {
            let mut x_s = x.slice_mut(0..3);
            x_s.add_scalar_mut(1);
        }

        assert_eq!(vec!(1, 1, 1, 0, 0, 0), x);
    }

    #[test]
    fn test_vec_norm() {
        let v = vec![3., -2., 6.];
        assert_eq!(v.norm(1.), 11.);
        assert_eq!(v.norm(2.), 7.);
        assert_eq!(v.norm(std::f64::INFINITY), 6.);
        assert_eq!(v.norm(std::f64::NEG_INFINITY), 2.);
    }

    #[test]
    fn test_vec_unique() {
        let n = vec![1, 2, 2, 3, 4, 5, 3, 2];
        assert_eq!(
            n.unique_with_indices(),
            (vec!(1, 2, 3, 4, 5), vec!(0, 1, 1, 2, 3, 4, 2, 1))
        );
        assert_eq!(n.unique(), vec!(1, 2, 3, 4, 5));
        assert_eq!(Vec::<i32>::zeros(100).unique(), vec![0]);
        assert_eq!(Vec::<i32>::zeros(100).slice(0..10).unique(), vec![0]);
    }

    #[test]
    fn test_vec_var_std() {
        assert_eq!(vec![1., 2., 3., 4., 5.].var(), 2.);
        assert_eq!(vec![1., 2.].std(), 0.5);
        assert_eq!(vec![1.].var(), 0.0);
        assert_eq!(vec![1.].std(), 0.0);
    }

    #[test]
    fn test_vec_abs() {
        let mut x = vec![-1, 2, -3];
        x.abs_mut();
        assert_eq!(x, vec![1, 2, 3]);
    }

    #[test]
    fn test_vec_neg() {
        let mut x = vec![-1, 2, -3];
        x.neg_mut();
        assert_eq!(x, vec![1, -2, 3]);
    }

    #[test]
    fn test_vec_copy_from() {
        let x = vec![1, 2, 3];
        let mut y = Vec::<i32>::zeros(3);
        y.copy_from(&x);
        assert_eq!(y, vec![1, 2, 3]);
    }

    #[test]
    fn test_vec_element_ops() {
        let mut x = vec![1, 2, 3, 4];
        x.slice_mut(0..1).mul_element_mut(0, 4);
        x.slice_mut(1..2).add_element_mut(0, 1);
        x.slice_mut(2..3).sub_element_mut(0, 1);
        x.slice_mut(3..4).div_element_mut(0, 4);
        assert_eq!(x, vec![4, 3, 2, 1]);
    }

    #[test]
    fn test_vec_ops() {
        assert_eq!(vec![1, 2, 3, 4].mul_scalar(2), vec![2, 4, 6, 8]);
        assert_eq!(vec![1, 2, 3, 4].add_scalar(2), vec![3, 4, 5, 6]);
        assert_eq!(vec![1, 2, 3, 4].sub_scalar(1), vec![0, 1, 2, 3]);
        assert_eq!(vec![1, 2, 3, 4].div_scalar(2), vec![0, 1, 1, 2]);
    }

    #[test]
    fn test_vec_init() {
        assert_eq!(Vec::<i32>::ones(3), vec![1, 1, 1]);
        assert_eq!(Vec::<i32>::zeros(3), vec![0, 0, 0]);
    }

    #[test]
    fn test_vec_min_max() {
        assert_eq!(ArrayView1::min(&vec![1, 2, 3, 4, 5, 6]), 1);
        assert_eq!(ArrayView1::max(&vec![1, 2, 3, 4, 5, 6]), 6);
    }

    #[test]
    fn test_vec_take() {
        assert_eq!(vec![1, 2, 3, 4, 5, 6].take(&[0, 4, 5]), vec![1, 5, 6]);
    }

    #[test]
    fn test_vec_rand() {
        let r = Vec::<f32>::rand(4);
        assert!(r.iterator(0).all(|&e| e <= 1f32));
        assert!(r.iterator(0).all(|&e| e >= 0f32));
        assert!(r.iterator(0).map(|v| *v).sum::<f32>() > 0f32);
    }

    #[test]
    #[should_panic]
    fn test_failed_vec_take() {
        assert_eq!(vec![1, 2, 3, 4, 5, 6].take(&[10, 4, 5]), vec![1, 5, 6]);
    }

    #[test]
    fn test_vec_quicksort() {
        let arr1 = vec![0.3, 0.1, 0.2, 0.4, 0.9, 0.5, 0.7, 0.6, 0.8];
        assert_eq!(vec![1, 2, 0, 3, 5, 7, 6, 8, 4], arr1.argsort());

        let arr2 = vec![
            0.2, 0.2, 0.2, 0.2, 0.2, 0.4, 0.3, 0.2, 0.2, 0.1, 1.4, 1.5, 1.5, 1.3, 1.5, 1.3, 1.6,
            1.0, 1.3, 1.4,
        ];
        assert_eq!(
            vec![9, 7, 1, 8, 0, 2, 4, 3, 6, 5, 17, 18, 15, 13, 19, 10, 14, 11, 12, 16],
            arr2.argsort()
        );
    }

    #[test]
    fn test_vec_map() {
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let expected = vec![2, 4, 6, 8];
        let result: Vec<i32> = a.map(|&v| v as i32 * 2);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_vec_mean() {
        let m = vec![1, 2, 3];

        assert_eq!(m.mean(), 2.0);
    }

    #[test]
    fn test_vec_max_diff() {
        let a = vec![1, 2, 3, 4, -5, 6];
        let b = vec![2, 3, 4, 1, 0, -12];
        assert_eq!(a.max_diff(&b), 18);
        assert_eq!(b.max_diff(&b), 0);
    }

    #[test]
    fn test_vec_softmax() {
        let mut prob = vec![1., 2., 3.];
        prob.softmax_mut();
        assert!((prob[0] - 0.09).abs() < 0.01);
        assert!((prob[1] - 0.24).abs() < 0.01);
        assert!((prob[2] - 0.66).abs() < 0.01);
    }

    #[test]
    fn test_min_max() {
        assert_eq!(
            DenseMatrix::from_2d_array(&[&[1, 2, 3], &[4, 5, 6]]).max(0),
            vec!(4, 5, 6)
        );
        assert_eq!(
            DenseMatrix::from_2d_array(&[&[1, 2, 3], &[4, 5, 6]]).max(1),
            vec!(3, 6)
        );
        assert_eq!(
            DenseMatrix::from_2d_array(&[&[1., 2., 3.], &[4., 5., 6.]]).min(0),
            vec!(1., 2., 3.)
        );
        assert_eq!(
            DenseMatrix::from_2d_array(&[&[1., 2., 3.], &[4., 5., 6.]]).min(1),
            vec!(1., 4.)
        );
    }

    #[test]
    fn test_argmax() {
        assert_eq!(
            DenseMatrix::from_2d_array(&[&[1, 5, 3], &[4, 2, 6]]).argmax(0),
            vec!(1, 0, 1)
        );
        assert_eq!(
            DenseMatrix::from_2d_array(&[&[4, 2, 3], &[1, 5, 6]]).argmax(1),
            vec!(0, 2)
        );
    }

    #[test]
    fn test_sum() {
        assert_eq!(
            DenseMatrix::from_2d_array(&[&[1, 2, 3], &[4, 5, 6]]).sum(0),
            vec!(5, 7, 9)
        );
        assert_eq!(
            DenseMatrix::from_2d_array(&[&[1., 2., 3.], &[4., 5., 6.]]).sum(1),
            vec!(6., 15.)
        );
    }

    #[test]
    fn test_abs() {
        let mut x = DenseMatrix::from_2d_array(&[&[-1, 2, -3], &[4, -5, 6]]);
        x.abs_mut();
        assert_eq!(x, DenseMatrix::from_2d_array(&[&[1, 2, 3], &[4, 5, 6]]));
    }

    #[test]
    fn test_neg() {
        let mut x = DenseMatrix::from_2d_array(&[&[-1, 2, -3], &[4, -5, 6]]);
        x.neg_mut();
        assert_eq!(x, DenseMatrix::from_2d_array(&[&[1, -2, 3], &[-4, 5, -6]]));
    }

    #[test]
    fn test_copy_from() {
        let x = DenseMatrix::from_2d_array(&[&[1, 2, 3], &[4, 5, 6]]);
        let mut y = DenseMatrix::<i32>::zeros(2, 3);
        y.copy_from(&x);
        assert_eq!(y, DenseMatrix::from_2d_array(&[&[1, 2, 3], &[4, 5, 6]]));
    }

    #[test]
    fn test_init() {
        let x = DenseMatrix::from_2d_array(&[&[1, 2, 3], &[4, 5, 6]]);
        assert_eq!(
            DenseMatrix::<i32>::zeros(2, 2),
            DenseMatrix::from_2d_array(&[&[0, 0], &[0, 0]])
        );
        assert_eq!(
            DenseMatrix::<i32>::ones(2, 2),
            DenseMatrix::from_2d_array(&[&[1, 1], &[1, 1]])
        );
        assert_eq!(
            DenseMatrix::<i32>::eye(3),
            DenseMatrix::from_2d_array(&[&[1, 0, 0], &[0, 1, 0], &[0, 0, 1]])
        );
        assert_eq!(
            DenseMatrix::from_slice(x.slice(0..2, 0..2).as_ref()),
            DenseMatrix::from_2d_array(&[&[1, 2], &[4, 5]])
        );
        assert_eq!(
            DenseMatrix::from_row(x.get_row(0).as_ref()),
            DenseMatrix::from_2d_array(&[&[1, 2, 3]])
        );
        assert_eq!(
            DenseMatrix::from_column(x.get_col(0).as_ref()),
            DenseMatrix::from_2d_array(&[&[1], &[4]])
        );
    }

    #[test]
    fn test_transpose() {
        let x = DenseMatrix::from_2d_array(&[&[1, 2, 3], &[4, 5, 6]]);
        assert_eq!(
            x.transpose(),
            DenseMatrix::from_2d_array(&[&[1, 4], &[2, 5], &[3, 6]])
        );
    }

    #[test]
    fn test_reshape() {
        let x = DenseMatrix::from_2d_array(&[&[1, 2, 3], &[4, 5, 6]]);
        assert_eq!(
            x.reshape(3, 2, 0),
            DenseMatrix::from_2d_array(&[&[1, 2], &[3, 4], &[5, 6]])
        );
        assert_eq!(
            x.reshape(3, 2, 1),
            DenseMatrix::from_2d_array(&[&[1, 4], &[2, 5], &[3, 6]])
        );
    }

    #[test]
    #[should_panic]
    fn test_failed_reshape() {
        let x = DenseMatrix::from_2d_array(&[&[1, 2, 3], &[4, 5, 6]]);
        assert_eq!(
            x.reshape(4, 2, 0),
            DenseMatrix::from_2d_array(&[&[1, 2], &[3, 4], &[5, 6]])
        );
    }

    #[test]
    fn test_matmul() {
        let a = DenseMatrix::from_2d_array(&[&[1, 2, 3], &[4, 5, 6]]);
        let b = DenseMatrix::from_2d_array(&[&[1, 2], &[3, 4], &[5, 6]]);
        assert_eq!(
            a.matmul(&(*b.slice(0..3, 0..2))),
            DenseMatrix::from_2d_array(&[&[22, 28], &[49, 64]])
        );
        assert_eq!(
            a.matmul(&b),
            DenseMatrix::from_2d_array(&[&[22, 28], &[49, 64]])
        );
    }

    #[test]
    fn test_concat() {
        let a = DenseMatrix::from_2d_array(&[&[1, 2], &[3, 4]]);
        let b = DenseMatrix::from_2d_array(&[&[5, 6], &[7, 8]]);

        assert_eq!(
            DenseMatrix::concatenate_1d(&[&vec!(1, 2, 3), &vec!(4, 5, 6)], 0),
            DenseMatrix::from_2d_array(&[&[1, 2, 3], &[4, 5, 6]])
        );
        assert_eq!(
            DenseMatrix::concatenate_1d(&[&vec!(1, 2), &vec!(3, 4)], 1),
            DenseMatrix::from_2d_array(&[&[1, 3], &[2, 4]])
        );
        assert_eq!(
            DenseMatrix::concatenate_2d(&[&a.clone(), &b.clone()], 0),
            DenseMatrix::from_2d_array(&[&[1, 2], &[3, 4], &[5, 6], &[7, 8]])
        );
        assert_eq!(
            DenseMatrix::concatenate_2d(&[&a, &b], 1),
            DenseMatrix::from_2d_array(&[&[1, 2, 5, 6], &[3, 4, 7, 8]])
        );
    }

    #[test]
    fn test_take() {
        let a = DenseMatrix::from_2d_array(&[&[1, 2, 3], &[4, 5, 6]]);
        let b = DenseMatrix::from_2d_array(&[&[1, 2], &[3, 4], &[5, 6]]);

        assert_eq!(
            a.take(&[0, 2], 1),
            DenseMatrix::from_2d_array(&[&[1, 3], &[4, 6]])
        );
        assert_eq!(
            b.take(&[0, 2], 0),
            DenseMatrix::from_2d_array(&[&[1, 2], &[5, 6]])
        );
    }

    #[test]
    fn test_merge() {
        let a = DenseMatrix::from_2d_array(&[&[1, 2], &[3, 4]]);

        assert_eq!(
            DenseMatrix::from_2d_array(&[&[1, 2], &[3, 4], &[5, 6], &[7, 8]]),
            a.merge_1d(&[&vec!(5, 6), &vec!(7, 8)], 0, true)
        );
        assert_eq!(
            DenseMatrix::from_2d_array(&[&[5, 6], &[7, 8], &[1, 2], &[3, 4]]),
            a.merge_1d(&[&vec!(5, 6), &vec!(7, 8)], 0, false)
        );
        assert_eq!(
            DenseMatrix::from_2d_array(&[&[1, 2, 5, 7], &[3, 4, 6, 8]]),
            a.merge_1d(&[&vec!(5, 6), &vec!(7, 8)], 1, true)
        );
        assert_eq!(
            DenseMatrix::from_2d_array(&[&[5, 7, 1, 2], &[6, 8, 3, 4]]),
            a.merge_1d(&[&vec!(5, 6), &vec!(7, 8)], 1, false)
        );
    }

    #[test]
    fn test_ops() {
        assert_eq!(
            DenseMatrix::from_2d_array(&[&[1, 2], &[3, 4]]).mul_scalar(2),
            DenseMatrix::from_2d_array(&[&[2, 4], &[6, 8]])
        );
        assert_eq!(
            DenseMatrix::from_2d_array(&[&[1, 2], &[3, 4]]).add_scalar(2),
            DenseMatrix::from_2d_array(&[&[3, 4], &[5, 6]])
        );
        assert_eq!(
            DenseMatrix::from_2d_array(&[&[1, 2], &[3, 4]]).sub_scalar(1),
            DenseMatrix::from_2d_array(&[&[0, 1], &[2, 3]])
        );
        assert_eq!(
            DenseMatrix::from_2d_array(&[&[1, 2], &[3, 4]]).div_scalar(2),
            DenseMatrix::from_2d_array(&[&[0, 1], &[1, 2]])
        );
    }

    #[test]
    fn test_rand() {
        let r = DenseMatrix::<f32>::rand(2, 2);
        assert!(r.iterator(0).all(|&e| e <= 1f32));
        assert!(r.iterator(0).all(|&e| e >= 0f32));
        assert!(r.iterator(0).map(|v| *v).sum::<f32>() > 0f32);
    }

    #[test]
    fn test_vstack() {
        let a = DenseMatrix::from_2d_array(&[&[1, 2, 3], &[4, 5, 6], &[7, 8, 9]]);
        let b = DenseMatrix::from_2d_array(&[&[1, 2, 3], &[4, 5, 6]]);
        let expected = DenseMatrix::from_2d_array(&[
            &[1, 2, 3],
            &[4, 5, 6],
            &[7, 8, 9],
            &[1, 2, 3],
            &[4, 5, 6],
        ]);
        let result = a.v_stack(&b);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_hstack() {
        let a = DenseMatrix::from_2d_array(&[&[1, 2, 3], &[4, 5, 6], &[7, 8, 9]]);
        let b = DenseMatrix::from_2d_array(&[&[1, 2], &[3, 4], &[5, 6]]);
        let expected =
            DenseMatrix::from_2d_array(&[&[1, 2, 3, 1, 2], &[4, 5, 6, 3, 4], &[7, 8, 9, 5, 6]]);
        let result = a.h_stack(&b);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_map() {
        let a = DenseMatrix::from_2d_array(&[&[1, 2, 3], &[4, 5, 6]]);
        let expected = DenseMatrix::from_2d_array(&[&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]]);
        let result: DenseMatrix<f64> = a.map(|&v| v as f64);
        assert_eq!(result, expected);
    }

    #[test]
    fn scale() {
        let mut m = DenseMatrix::from_2d_array(&[&[1., 2., 3.], &[4., 5., 6.]]);
        let expected_0 = DenseMatrix::from_2d_array(&[&[-1., -1., -1.], &[1., 1., 1.]]);
        let expected_1 = DenseMatrix::from_2d_array(&[&[-1.22, 0.0, 1.22], &[-1.22, 0.0, 1.22]]);

        {
            let mut m = m.clone();
            m.scale_mut(&m.mean(0), &m.std(0), 0);
            assert!(relative_eq!(m, expected_0));
        }

        m.scale_mut(&m.mean(1), &m.std(1), 1);
        assert!(relative_eq!(m, expected_1, epsilon = 1e-2));
    }

    #[test]
    fn test_pow_mut() {
        let mut a = DenseMatrix::from_2d_array(&[&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]]);
        a.pow_mut(2.0);
        assert_eq!(
            a,
            DenseMatrix::from_2d_array(&[&[1.0, 4.0, 9.0], &[16.0, 25.0, 36.0]])
        );
    }
}